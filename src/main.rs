use std::collections::HashSet;
use std::io::{Read, Write};
use std::fs::File;
use std::path::Path;
use reqwest::blocking::{Client, Response};
use select::{document::Document, predicate::Name};

const ORIGIN_URL: &str = "https://old.reddit.com";

fn main() -> std::io::Result<()>{
    let file = init_file("csv/crawl.csv".to_string());
    let mut urls_to_crawl = HashSet::new();
    let mut visited_urls = HashSet::new();

    urls_to_crawl.insert(ORIGIN_URL.to_string());

    while urls_to_crawl.len() > 0 {
        let mut urls_to_add = HashSet::new();
        println!("{} URLs to crawl", urls_to_crawl.len());
        for url in &urls_to_crawl {
            let response: Response;
            let res = get_page(url.to_string());

            match res {
                None => continue,
                _ => response = res.unwrap(),
            }

            match format_url(url.to_string()) {
                None => continue,
                url => visited_urls.insert(url.unwrap()),
            };

            write_response_to_file(&file, &response);

            for url in &get_pages_to_crawl(response) {
                match format_url(url.to_string()) {
                    None => continue,
                    formated_url => {
                        let url = formated_url.unwrap();
                        if !visited_urls.contains(&url) {
                            urls_to_add.insert(url);
                        }
                    },
                };
            }
        }
        urls_to_crawl.clear();

        urls_to_crawl = urls_to_add.difference(&visited_urls)
            .cloned()
            .collect::<HashSet<String>>();

        println!("urls to add {}", urls_to_add.len());
        println!("visited url {}", visited_urls.len());
        println!("urls to crawl {}", urls_to_crawl.len());
    }
    Ok(())
}

fn get_page(mut url: String) -> Option<Response> {

    url = format_url(url).unwrap();

    let client = Client::new();
    let res = client.get(url).send().unwrap();

    let content_type = res.headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();

    if !content_type.contains("text/html") {
        return None;
    }

    return Some(res);
}

fn get_pages_to_crawl(mut res: Response) -> HashSet<String> {
    let mut body  = String::new();
    res.read_to_string(&mut body).unwrap();

    let found_urls = Document::from(body.as_str())
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .map(str::to_string)
        .collect::<HashSet<String>>();

    return found_urls;
}

fn init_file(filename: String) -> File {
    let path = Path::new(&filename);

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", filename, why),
        Ok(file) => file,
    };

    match file.write_all(b"URL,Status_Code\n") {
        Err(error) => panic!("Couldn't write to {}: {}", filename, error),
        Ok(_) => (),
    }
    return file;
}

fn write_response_to_file(mut file: &File, response: &Response) {
    match file.write_all(response.url().to_string().as_bytes()) {
        Err(error) => panic!("Couldn't write to file : {}", error),
        Ok(_) => (),
    }
    match file.write_all(b",") {
        Err(error) => panic!("Couldn't write to file : {}", error),
        Ok(_) => (),
    }
    match file.write_all(response.status().as_str().as_bytes()) {
        Err(error) => panic!("Couldn't write to file : {}", error),
        Ok(_) => (),
    }
    match file.write_all(b"\n") {
        Err(error) => panic!("Couldn't write to file : {}", error),
        Ok(_) => (),
    }
}

fn format_url(url: String) -> Option<String> {
    if url.starts_with('/') {
        return Some(ORIGIN_URL.to_owned() + &url);
    }
    else if url.starts_with("#") {
        return None;
    }
    else if !url.starts_with(ORIGIN_URL) {
        return None;
    }
    else {
        return Some(url);
    }
}
