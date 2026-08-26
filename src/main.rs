use avocalc::*;
use std::time::Instant;
use gl_headless::gl_headless;

// Более сложное выражение для нагрузки
const EXPR: &str = "TexCoord.x * TexCoord.y + sin(TexCoord.x * 50.0) * cos(TexCoord.y * 50.0) + pow(TexCoord.x, 3.0) - pow(TexCoord.y, 2.0)";
const ITERATIONS: usize = 100;   // количество повторений для усреднения

#[gl_headless]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Сохраняем выражение в глобальный реестр
    {
        let mut map = Expressions.lock().unwrap();
        map.insert("test".to_string(), EXPR.to_string());
    }

    // 2. Компилируем шейдеры (делаем это один раз до замера)
    Compile_shader("test")?;

    // 3. Замер времени на GPU (только выполнение, без компиляции)
    let start_gpu = Instant::now();
    let mut gpu_texture = Vec::new();
    for _ in 0..ITERATIONS {
        gpu_texture = Calculation_to_texture("test")?; // размер 256x256, формат f32
    }
    let duration_gpu = start_gpu.elapsed();

    // 4. Замер времени на CPU (аналогичное количество вычислений)
    let width = 256;
    let height = 256;
    let total_pixels = width * height;

    let start_cpu = Instant::now();
    let mut cpu_last = Vec::with_capacity(total_pixels);
    for iter in 0..ITERATIONS {
        let mut cpu_results = Vec::with_capacity(total_pixels);
        for y in 0..height {
            for x in 0..width {
                // Координаты texel-центра (как в OpenGL)
                let tx = (x as f32 + 0.5) / width as f32;
                let ty = (y as f32 + 0.5) / height as f32;
                let val = tx * ty + (tx * 50.0).sin() * (ty * 50.0).cos() + tx.powi(3) - ty.powi(2);
                cpu_results.push(val);
            }
        }
        // Сохраняем результат последней итерации для сравнения
        if iter == ITERATIONS - 1 {
            cpu_last = cpu_results;
        }
    }
    let duration_cpu = start_cpu.elapsed();

    // 5. Сравнение результатов (последняя итерация)
    let mut max_diff = 0.0f32;
    let mut sum_diff = 0.0f32;
    for (&gpu, &cpu) in gpu_texture.iter().zip(cpu_last.iter()) {
        let diff = (gpu - cpu).abs();
        if diff > max_diff {
            max_diff = diff;
        }
        sum_diff += diff;
    }
    let avg_diff = sum_diff / (total_pixels as f32);

    // 6. Вывод статистики
    println!("Expression: {}", EXPR);
    println!("Resolution: {}x{}", width, height);
    println!("Iterations: {}", ITERATIONS);
    println!("GPU total time: {:?} (avg per frame: {:?})",
             duration_gpu, duration_gpu / ITERATIONS as u32);
    println!("CPU total time: {:?} (avg per frame: {:?})",
             duration_cpu, duration_cpu / ITERATIONS as u32);
    println!("Max diff: {:.6e}", max_diff);
    println!("Avg diff: {:.6e}", avg_diff);

    Ok(())
}