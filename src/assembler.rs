use std::collections::HashMap;

struct Error {}

const EMPTY_LINE: &str = "0000000000000000";

pub fn assemble(assembly_code: Vec<String>) -> Vec<String> {
    let mut symbol_table = build_symbol_table();
    //needs comment ehre
    let mut line_numbers = Vec::<(usize, String)>::new();
    let mut machine_code = Vec::<String>::new();

    //After this first pass, we might have the full translation apart from
    //symbols used earlier in the file than they are declared, and variables.
    assemble_first_pass(
        &mut symbol_table,
        assembly_code,
        &mut machine_code,
        &mut line_numbers,
    );

    //Now we can revisit the lines we could not translate due to missing information
    assemble_missing_lines(&mut symbol_table, &mut machine_code, line_numbers);

    machine_code
}

fn assemble_first_pass(
    symbol_table: &mut HashMap<String, usize>,
    assembly_code: Vec<String>,
    machine_code: &mut Vec<String>,
    line_numbers: &mut Vec<(usize, String)>,
) {
    let mut line_no = 0;

    //After this first pass, we might have the full translation apart from
    //symbols used earlier in the file than they are declared, and variables.
    for line in assembly_code.iter() {
        let trimmed_line = line.trim_start();

        if let Some(first_char) = trimmed_line.chars().next() {
            //Ignore comments
            if first_char == '/' {
            }
            //Symbol definitions
            else if first_char == '(' {
                let clean_key = &trimmed_line[1..trimmed_line.len() - 1];
                symbol_table.insert(clean_key.to_string(), line_no);
            //A instructions
            } else if first_char == '@' {
                match assemble_a_instruction(&trimmed_line, &symbol_table) {
                    Ok(translated_line) => machine_code.push(translated_line),
                    Err(_) => {
                        line_numbers.push((line_no, trimmed_line.to_string()));
                        machine_code.push(EMPTY_LINE.to_string())
                    }
                };
                line_no = line_no + 1;
            //c instructions
            } else {
                let translated_line = assemble_c_instruction(&trimmed_line);
                machine_code.push(translated_line);
                line_no = line_no + 1;
            }
        }
    }
}

fn assemble_missing_lines(
    symbol_table: &mut HashMap<String, usize>,
    machine_code: &mut Vec<String>,
    line_numbers: Vec<(usize, String)>,
) {
    let mut next_var_address = 16;

    for (line_no, line) in line_numbers.iter() {
        match assemble_a_instruction(&line, &symbol_table) {
            Ok(translated_line) => machine_code[*line_no] = translated_line,
            Err(_) => {
                let trimmed_line = line.trim_start_matches("@");
                symbol_table.insert(trimmed_line.to_string(), next_var_address);
                machine_code[*line_no] = format!("{:016b}", next_var_address);
                next_var_address = next_var_address + 1;
            }
        }
    }
}

pub fn build_symbol_table() -> HashMap<String, usize> {
    let mut symbol_table = HashMap::<String, usize>::new();

    for i in 0..16 {
        symbol_table.insert(format!("R{i}"), i);
    }

    symbol_table.insert("SP".to_string(), 0);
    symbol_table.insert("LCL".to_string(), 1);
    symbol_table.insert("ARG".to_string(), 2);
    symbol_table.insert("THIS".to_string(), 3);
    symbol_table.insert("THAT".to_string(), 4);

    symbol_table.insert("SCREEN".to_string(), 16384);
    symbol_table.insert("KBD".to_string(), 24576);

    symbol_table
}

fn assemble_a_instruction(
    a_instr: &str,
    symbol_table: &HashMap<String, usize>,
) -> Result<String, Error> {
    let trimmed_line = a_instr.trim_start_matches("@");

    //If parsing fails, we know it is referring to a symbol
    match trimmed_line.parse::<u16>() {
        Ok(address_no) => Ok(format!("{:016b}", address_no)),
        Err(_err) => match symbol_table.get(trimmed_line) {
            Some(address_no) => Ok(format!("{:016b}", address_no)),
            None => Err(Error {}),
        },
    }
}

fn assemble_c_instruction(line: &str) -> String {
    let mut result = String::new();
    let mut comp = String::new();
    let mut dest = String::new();
    let mut jmp = String::new();
    let mut acc = String::new();

    dest.push_str("000");
    result.push_str("111");
    jmp.push_str("000");

    for (i, c) in line.chars().enumerate() {
        if c == '=' {
            dest.clear();
            dest.push_str(assemble_dest(&acc));
            acc.clear();
        } else if c == ';' {
            comp.push_str(assemble_comp(&acc));
            acc.clear();
        } else if i == line.chars().count() - 1 {
            acc.push(c);
            if comp.is_empty() {
                comp.push_str(assemble_comp(&acc));
            } else {
                jmp.clear();
                jmp.push_str(assemble_jmp(&acc));
            }
        } else {
            acc.push(c);
        }
    }

    result.push_str(&comp);
    result.push_str(&dest);
    result.push_str(&jmp);

    result
}

fn assemble_dest(dest: &str) -> &str {
    match dest {
        "M" => "001",
        "D" => "010",
        "MD" => "011",
        "A" => "100",
        "AM" => "101",
        "AD" => "110",
        "AMD" => "111",
        dest => panic!("unknown dest {dest}"),
    }
}

fn assemble_jmp(jmp: &str) -> &str {
    match jmp {
        "JGT" => "001",
        "JEQ" => "010",
        "JGE" => "011",
        "JLT" => "100",
        "JNE" => "101",
        "JLE" => "110",
        "JMP" => "111",
        jmp => panic!("unknown jmp {jmp}"),
    }
}

fn assemble_comp(comp: &str) -> &str {
    match comp {
        //a bit unset
        "0" => "0101010",
        "1" => "0111111",
        "-1" => "0111010",
        "D" => "0001100",
        "A" => "0110000",
        "!D" => "0001101",
        "!A" => "0110001",
        "-D" => "0001111",
        "-A" => "0110011",
        "D+1" => "0011111",
        "A+1" => "0110111",
        "D-1" => "0001110",
        "A-1" => "0110010",
        "D+A" => "0000010",
        "D-A" => "0010011",
        "A-D" => "0000111",
        "D&A" => "0000000",
        "A|D" => "0010101",
        //a bit set
        "M" => "1110000",
        "!M" => "1110001",
        "-M" => "1110011",
        "M+1" => "1110111",
        "M-1" => "1110010",
        "D+M" => "1000010",
        "D-M" => "1010011",
        "M-D" => "1000111",
        "D&M" => "1000000",
        "D|M" => "1010101",
        comp => panic!("unknown {comp}"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_max() {
        let file = read_file("test_files/MaxL.asm");
        let result = assemble(file);
        let expected_result = read_file("test_files/MaxL.hack");
        assert_eq!(result, expected_result);
    }

    #[test]
    fn translate_max_symbols() {
        let file = read_file("test_files/Max.asm");
        let result = assemble(file);
        let expected_result = read_file("test_files/Max.hack");
        assert_eq!(result, expected_result);
    }

    #[test]
    fn translate_pong() {
        let file = read_file("test_files/PongL.asm");
        let result = assemble(file);
        let expected_result = read_file("test_files/PongL.hack");
        assert_eq!(result, expected_result);
    }

    #[test]
    fn translate_pong_symbols() {
        let file = read_file("test_files/Pong.asm");
        let result = assemble(file);
        let expected_result = read_file("test_files/Pong.hack");
        assert_eq!(result, expected_result);
    }

    fn read_file(filename: &str) -> Vec<String> {
        std::fs::read_to_string(filename)
            .unwrap()
            .lines()
            .map(String::from)
            .collect()
    }
}
