use crate::utils::errors::CustomError;
use base64::{engine::general_purpose, Engine as _};
use image::{load_from_memory, ImageBuffer, ImageFormat, Luma};
use std::error::Error;
use std::fs::File;
use std::io::{Cursor, Write};

pub fn get_ocr_result_from_b64(b64_string: String) -> Result<String, Box<dyn Error + Send>> {
    classification_image(binarize_image(base64_to_u8(b64_string)))
}

pub fn base64_to_u8(b64_string: String) -> Vec<u8> {
    let bytes = general_purpose::STANDARD.decode(b64_string).unwrap();
    bytes
}

pub fn binarize_image(image: Vec<u8>) -> Vec<u8> {
    // 将图像转换为灰度图
    let gray_image = load_from_memory(&image).unwrap().to_luma8();
    // 创建一个新的图像缓冲区用于存储二值化后的图像
    let mut binarized_image = ImageBuffer::new(gray_image.width(), gray_image.height());

    // 设置二值化的阈值
    let threshold = 128;

    // 遍历每个像素，根据阈值进行二值化
    for (x, y, pixel) in gray_image.enumerate_pixels() {
        let luma = pixel[0]; // 灰度图的亮度值
        let binary_value = if luma > threshold { 255 as u8 } else { 0 as u8 };
        binarized_image.put_pixel(x, y, Luma([binary_value]));
    }

    // binarized_image.save("puzzle.png").unwrap();

    let mut png_data = Vec::new();

    let mut corsor = Cursor::new(&mut png_data);
    binarized_image
        .write_to(&mut corsor, ImageFormat::Png)
        .unwrap();

    // 创建或打开文件
    let mut file = File::create("puzzle.png").unwrap();

    // 将 Vec<u8> 写入文件s
    file.write_all(&mut png_data).unwrap();
    png_data
}

pub fn classification_image(png_data: Vec<u8>) -> Result<String, Box<dyn Error + Send>> {
    let mut ocr = ddddocr::ddddocr_classification_old().unwrap();
    let ans = ocr
        .classification(png_data, true)
        .map_err(|e| CustomError::CaptchaError(e.to_string()));
    if ans.is_err() {
        return Err(Box::new(ans.unwrap_err()));
    }
    Ok(ans.unwrap())
}
