use encoding_rs::ISO_8859_10;
use std::error::Error;
extern crate scraper;
use crate::utils::errors::CustomError::HtmlParseError;
use scraper::{Html, Selector};

pub async fn find_and_return_regist_url(
    html: String,
    selectors: &str,
) -> Result<String, Box<dyn Error + Send>> {
    // 转编码
    let bytes_html = html.as_bytes();
    let (utf8_string, _, had_errors) = ISO_8859_10.decode(bytes_html);

    if had_errors {
        println!("Encoding conversion had errors");
        return Err(Box::new(HtmlParseError(
            "Encoding conversion error".to_string(),
        )));
    }

    // 解析 HTML
    let document = Html::parse_document(&utf8_string);

    // 使用 CSS 选择器提取元素
    let selector = Selector::parse(selectors).unwrap();
    for element in document.select(&selector) {
        println!("元素文本：{}", element.inner_html());
        return Ok(element.attr("href").unwrap().to_string());
    }
    // let element = document.select(&selector);
    return Err(Box::new(HtmlParseError("No element found".to_string())));

    // // 解析 XML 文档
    // let package = parser::parse(&utf8_string).expect("Failed to parse XML");
    // let document = package.as_document();

    // // 查询标题为 "Rust Programming" 的书籍作者
    // let xpath_expr = &xpath;

    // // 执行 XPath 查询
    // match evaluate_xpath(&document, xpath_expr) {
    //     Ok(Value::Nodeset(nodeset)) => {
    //         return Ok("abcx".to_string());
    //     }
    //     Ok(_) => {
    //         println!("No results found.");
    //         return Err(Box::new(HtmlParseError(
    //             "Xpath No results found.".to_string(),
    //         )));
    //     }
    //     Err(e) => {
    //         return Err(Box::new(HtmlParseError(format!("XPath error: {}", e))));
    //     }
    // }
}
