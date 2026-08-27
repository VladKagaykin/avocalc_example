use avocalc::*;
use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

// ---------- Вычисления на CPU (парсер) ----------
fn eval_cpu(expr: &str, vars: &HashMap<String, f32>) -> Result<f32, String> {
    fn extract_vars(expr: &str) -> Result<(Vec<String>, String), String> {
        let mut vars = Vec::new();
        let mut result = String::new();
        let mut chars = expr.chars().peekable();
        let mut brace_count = 0;
        while let Some(c) = chars.next() {
            match c {
                '{' => {
                    brace_count += 1;
                    let mut name = String::new();
                    let mut closed = false;
                    while let Some(&next) = chars.peek() {
                        if next == '}' {
                            chars.next();
                            brace_count -= 1;
                            closed = true;
                            break;
                        } else {
                            name.push(chars.next().unwrap());
                        }
                    }
                    if !closed {
                        return Err("Unclosed '{'".to_string());
                    }
                    if name.is_empty() {
                        return Err("Empty variable".to_string());
                    }
                    if !name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
                        return Err(format!("Invalid var name '{}'", name));
                    }
                    vars.push(name.clone());
                    result.push_str(&name);
                }
                '}' => return Err("Unexpected '}'".to_string()),
                _ => result.push(c),
            }
        }
        if brace_count != 0 {
            return Err("Unbalanced braces".to_string());
        }
        Ok((vars, result))
    }

    let (_, replaced) = extract_vars(expr)?;
    let chars: Vec<char> = replaced.chars().filter(|c| !c.is_whitespace()).collect();
    let mut pos = 0;

    fn parse_expr(chars: &[char], pos: &mut usize, vars: &HashMap<String, f32>) -> Result<f32, String> {
        let mut left = parse_term(chars, pos, vars)?;
        while *pos < chars.len() {
            let op = chars[*pos];
            if op == '+' || op == '-' {
                *pos += 1;
                let right = parse_term(chars, pos, vars)?;
                left = if op == '+' { left + right } else { left - right };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_term(chars: &[char], pos: &mut usize, vars: &HashMap<String, f32>) -> Result<f32, String> {
        let mut left = parse_factor(chars, pos, vars)?;
        while *pos < chars.len() {
            let op = chars[*pos];
            if op == '*' || op == '/' {
                *pos += 1;
                let right = parse_factor(chars, pos, vars)?;
                left = if op == '*' { left * right } else { left / right };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_factor(chars: &[char], pos: &mut usize, vars: &HashMap<String, f32>) -> Result<f32, String> {
        if *pos >= chars.len() {
            return Err("Unexpected end".to_string());
        }
        let ch = chars[*pos];
        if ch == '(' {
            *pos += 1;
            let val = parse_expr(chars, pos, vars)?;
            if *pos < chars.len() && chars[*pos] == ')' {
                *pos += 1;
                Ok(val)
            } else {
                Err("Missing ')'".to_string())
            }
        } else if ch.is_ascii_digit() || ch == '.' {
            let start = *pos;
            while *pos < chars.len() && (chars[*pos].is_ascii_digit() || chars[*pos] == '.') {
                *pos += 1;
            }
            let num: String = chars[start..*pos].iter().collect();
            num.parse::<f32>().map_err(|_| format!("Invalid number '{}'", num))
        } else if ch.is_ascii_alphabetic() || ch == '_' {
            let start = *pos;
            while *pos < chars.len() && (chars[*pos].is_ascii_alphanumeric() || chars[*pos] == '_') {
                *pos += 1;
            }
            let name: String = chars[start..*pos].iter().collect();
            vars.get(&name).copied().ok_or_else(|| format!("Variable '{}' not found", name))
        } else {
            Err(format!("Unexpected char '{}'", ch))
        }
    }

    parse_expr(&chars, &mut pos, vars)
}

// ---------- Демонстрация ----------
fn main() -> Result<(), Box<dyn Error>> {
    {
        let mut expr_map = Expressions.lock().unwrap();
        expr_map.clear();
        let mut shader_obj_map = Shader_objects.lock().unwrap();
        shader_obj_map.clear();
    }

    let name1 = "example1";
    let expr1 = "{x} * 2.0 + 3.5";
    {
        let mut map = Expressions.lock().unwrap();
        map.insert(name1.to_string(), expr1.to_string());
        let mut map2 = Shader_objects.lock().unwrap();
        map2.insert("x".to_string(), "2.0".to_string());
    }
    Compile_shader(name1)?;
    let res1 = Calculation_to_texture(name1)?;
    println!("1) {} = {:.3}", expr1, res1);

    let name = "complex";
    let expr = "(({a}+{b})*({c}-{d}) + ({e}*{f})/({g}+{h})) * {i} - {j}";
    {
        let mut map = Expressions.lock().unwrap();
        map.insert(name.to_string(), expr.to_string());
        let mut map2 = Shader_objects.lock().unwrap();
        map2.insert("a".to_string(), "1.2".to_string());
        map2.insert("b".to_string(), "3.4".to_string());
        map2.insert("c".to_string(), "5.6".to_string());
        map2.insert("d".to_string(), "7.8".to_string());
        map2.insert("e".to_string(), "9.0".to_string());
        map2.insert("f".to_string(), "1.5".to_string());
        map2.insert("g".to_string(), "2.5".to_string());
        map2.insert("h".to_string(), "3.5".to_string());
        map2.insert("i".to_string(), "4.5".to_string());
        map2.insert("j".to_string(), "6.7".to_string());
    }

    Compile_shader(name)?;

    let vars: HashMap<String, f32> = {
        let map = Shader_objects.lock().unwrap();
        map.iter()
            .map(|(k, v)| (k.clone(), v.parse::<f32>().unwrap()))
            .collect()
    };

    const N: usize = 1_000_000;
    const CHUNK_SIZE: usize = 4096;  // уменьшено с 32768 до 4096

    let start_gpu = Instant::now();
    let mut results_gpu = Vec::with_capacity(N);
    let mut processed = 0;
    while processed < N {
        let chunk = std::cmp::min(CHUNK_SIZE, N - processed);
        for _ in 0..chunk {
            add_calculation(name)?;
        }
        let chunk_results = calculate_texture()?;
        if chunk_results.len() != chunk {
            return Err(format!(
                "Ожидалось {} результатов, получено {}",
                chunk,
                chunk_results.len()
            )
            .into());
        }
        results_gpu.extend(chunk_results);
        processed += chunk;
    }
    let dur_gpu = start_gpu.elapsed();

    let start_cpu = Instant::now();
    let mut results_cpu = Vec::with_capacity(N);
    for _ in 0..N {
        let val = eval_cpu(expr, &vars)?;
        results_cpu.push(val);
    }
    let dur_cpu = start_cpu.elapsed();

    println!("\n--- Сравнение GPU vs CPU (пакетная обработка, {} элементов) ---", N);
    println!("Выражение: {}", expr);
    if results_gpu.len() == N && results_cpu.len() == N {
        println!("Первые 5 результатов:");
        for i in 0..5.min(N) {
            println!("  [{}] GPU: {:.6}, CPU: {:.6}", i, results_gpu[i], results_cpu[i]);
        }
        let all_match = results_gpu
            .iter()
            .zip(results_cpu.iter())
            .all(|(g, c)| (g - c).abs() < 1e-5);
        if all_match {
            println!("Результаты совпадают (в пределах погрешности).");
        } else {
            println!("Результаты НЕ совпадают (возможны погрешности).");
        }
    }

    println!("Общее время GPU: {:?}  (среднее на элемент: {:?})", dur_gpu, dur_gpu / N as u32);
    println!("Общее время CPU: {:?}  (среднее на элемент: {:?})", dur_cpu, dur_cpu / N as u32);
    println!("Ускорение GPU: {:.2}x", dur_cpu.as_secs_f64() / dur_gpu.as_secs_f64());

    Ok(())
}