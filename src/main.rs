use avocalc::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Очистка публичных карт (опционально)
    {
        let mut expr_map = Expressions.lock().unwrap();
        expr_map.clear();
        let mut shader_obj_map = Shader_objects.lock().unwrap();
        shader_obj_map.clear();
    }

    // Пример 1: одна переменная
    let name1 = "example1";
    let expr1 = "{x} * 2.0 + 3.5";
    {
        let mut map = Expressions.lock().unwrap();
        map.insert(name1.to_string(), expr1.to_string());
    }
    {
        let mut map = Shader_objects.lock().unwrap();
        map.insert("x".to_string(), "2.0".to_string());
    }

    Compile_shader(name1)?;
    let result1 = Calculation_to_texture(name1)?;
    println!("1) '{}' при x = 2.0  →  {:.3}", expr1, result1);

    // Пример 2: несколько переменных
    let name2 = "example2";
    let expr2 = "{a} * {b} + {c}";
    {
        let mut map = Expressions.lock().unwrap();
        map.insert(name2.to_string(), expr2.to_string());
    }
    {
        let mut map = Shader_objects.lock().unwrap();
        map.insert("a".to_string(), "1.5".to_string());
        map.insert("b".to_string(), "2.0".to_string());
        map.insert("c".to_string(), "0.5".to_string());
    }

    Compile_shader(name2)?;
    let result2 = Calculation_to_texture(name2)?;
    println!("2) '{}' при a=1.5, b=2.0, c=0.5  →  {:.3}", expr2, result2);

    // Пример 3: обработка ошибки (отсутствует uniform)
    let name3 = "example3";
    let expr3 = "{missing} * 2.0";
    {
        let mut map = Expressions.lock().unwrap();
        map.insert(name3.to_string(), expr3.to_string());
    }
    // НЕ добавляем "missing" в Shader_objects

    Compile_shader(name3)?;
    match Calculation_to_texture(name3) {
        Ok(v) => println!("3) Неожиданный успех: {}", v),
        Err(e) => println!("3) Ожидаемая ошибка: {}", e),
    }

    // Пример 4: целочисленный литерал (используем 10.0 вместо 10)
    let name4 = "example4";
    let expr4 = "{y} + 10.0";  // <-- изменено с 10 на 10.0
    {
        let mut map = Expressions.lock().unwrap();
        map.insert(name4.to_string(), expr4.to_string());
    }
    {
        let mut map = Shader_objects.lock().unwrap();
        map.insert("y".to_string(), "5.0".to_string());
    }

    Compile_shader(name4)?;
    let result4 = Calculation_to_texture(name4)?;
    println!("4) '{}' при y = 5.0  →  {:.3}", expr4, result4);

    Ok(())
}