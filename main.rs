use std::process::{Command,Stdio};
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;
use std::thread::sleep;
use std::sync::Mutex;

use reqwest::header::{ACCEPT, HeaderValue, USER_AGENT, ACCEPT_LANGUAGE, ACCEPT_ENCODING, HeaderName};
use reqwest::{Client,Proxy, header};
use regex::Regex;
use select::document::Document;
use select::predicate::{Name, Attr};
use sqlite::Connection;

static SEARCHED_LIST: Mutex<Vec<String>> = Mutex::new(Vec::new());

async fn add_sql(title:String,description:String,url:String) {
    let conn = Connection::open("./onionlist.db").unwrap();
    let result = conn.execute(format!("INSERT INTO site(url,title,description) values('{url}' , '{title}' , '{description}');").to_string());
    match result {
        Ok(_) => {},
        Err(_) => {},
    }
}

async fn search_sql(url: &String) -> String {
    let mut result = "err";
    let searched_list = SEARCHED_LIST.lock().unwrap();
    let result_ = searched_list.iter().find(|&item| item == &url.to_string());
    match result_ {
        Some(_) => {
            result = "find";
        },
        None => {},
    }
    return result.to_string();
}

async fn html_parse(content: String, url: String) -> Vec<String> {
    let mut url_list:Vec<String> = vec![];

    let mut title = String::from("null");
    let mut description = String::new();
    _ = description;
    let document = Document::from_read(content.as_bytes()).expect("Err HTML Docs");
    if let Some(title_node) = document.find(Name("title")).next() {
        title = title_node.text();
    }
    let description_node = Document::from_read(content.as_bytes())
        .expect("Failed to parse HTML")
        .find(Attr("name", "description"))
        .filter_map(|meta_node| meta_node.attr("content").map(String::from))
        .next();
    match description_node {
        Some(content) => description = String::from(content),
        None => description = String::from("null"),
    }


    let url_template = Regex::new(r"http://[A-Za-z0-9.]+.onion").unwrap();
    for mat in url_template.find_iter(&content) {
        let result = url_list.iter().find(|&item| item == &mat.as_str().to_string());
        match result {
            Some(_) => {},
            None => url_list.push(mat.as_str().to_string()),
        }
    };
    println!("============================================================================================");
    println!("Title: {}\nDescription: {}\nURL: {}\n{}個の新規URLを取得",title,description,url,&url_list.len().to_string());
    println!("============================================================================================");
    add_sql(title, description, url).await;
    return url_list;
}

async fn get_request() {
    println!("IP確認");
    let start_url = String::from("http://paavlaytlfsqyvkg3yqj7hflfg5jw2jdg2fgkza5ruf6lplwseeqtvyd.onion");
    let proxy = Proxy::all("socks5://127.0.0.1:9050").expect("err");
    let client = Client::builder()
        .proxy(proxy)
        .build();
    let response = client.expect("Err").get("https://httpbin.org/ip").send().await;
    println!("{}", response.expect("Err").text().await.unwrap());
    println!("Tor Crawlerを起動します");
    let mut url_list1:Vec<String> = onion_get_request(start_url).await;
    let mut url_list2:Vec<String> = vec![];
    url_list1.remove(0);
    loop {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open("./log.txt")
            .expect("FILE FUCK");
        for url in url_list1.iter() {
            println!("接続開始:{}",&url.to_string());
            let get_result = onion_get_request(url.to_string()).await;
            for urls in get_result.iter() {
                let search = url_list2.iter().find(|&item| &item == &urls);
                match search {
                    Some(_) => {},
                    None => url_list2.push(urls.to_string()),
                }
            }
        }
        url_list1 = vec![];
        println!("リストチェンジ:{}",url_list2.len().to_string());
        for fuck in url_list2.iter() {
            match file.write_all(format!("\n{fuck}").as_bytes()) {
                Ok(_) => {},
                Err(_) => {},
            }
        }

        for url in url_list2.iter() {
            println!("接続開始:{}",&url.to_string());
            let get_result = onion_get_request(url.to_string()).await;
            for urls in get_result.iter() {
                let search = url_list1.iter().find(|&item| &item == &urls);
                match search {
                    Some(_) => {},
                    None => url_list1.push(urls.to_string()),
                }
            }
        }
        url_list2 = vec![];
        println!("リストチェンジ2:{}",url_list1.len().to_string());
        for fuck in url_list1.iter() {
            match file.write_all(format!("\n{fuck}").as_bytes()) {
                Ok(_) => {},
                Err(_) => {},
            }
        }

        let lenght1:String = url_list1.len().to_string();
        let lenght2:String = url_list2.len().to_string();
        if lenght1 == "0" && lenght2 == "0" {
            break;
        }
    }
    println!("終了")
}

async fn onion_get_request(url: String) -> Vec<String> {
    let mut url_list:Vec<String> = vec![];
    if "find".to_string() == search_sql(&url).await {
        return url_list;
    } else {
        SEARCHED_LIST.lock().unwrap().push(url.to_string());
        let mut headers = header::HeaderMap::new();
        headers.insert(ACCEPT,HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"));
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; rv:91.0) Gecko/20100101 Firefox/91.0"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
        headers.insert(HeaderName::from_static("Sec-Fetch-Dest"),HeaderValue::from_str("document").unwrap());
        headers.insert(HeaderName::from_static("Sec-Fetch-Mode"),HeaderValue::from_str("navigate").unwrap());
        headers.insert(HeaderName::from_static("Sec-Fetch-Site"),HeaderValue::from_str("none").unwrap());
        headers.insert(HeaderName::from_static("Sec-Fetch-User"),HeaderValue::from_str("?1").unwrap());
        headers.insert(HeaderName::from_static("Upgrade-Insecure-Requests"),HeaderValue::from_str("1").unwrap());
        let proxy = Proxy::all("socks5h://127.0.0.1:9050").expect("err");
        let client = Client::builder()
            .proxy(proxy)
            .timeout(Duration::from_secs(40))
            .default_headers(headers)
            .build();
        let response = client.expect("Err").get(&url).send().await;
        match response {
            Ok(_) => {
                match response.expect("Err").text().await {
                    Ok(body) => {
                        url_list = html_parse(body, url).await;
                        return url_list;
                    },
                    Err(_) => {
                        println!("接続停止:{}",&url.to_string());
                        println!("============================================================================================");
                        return url_list;
                    },
                }
            },
            Err(_) => {
                println!("接続停止:{}",&url.to_string());
                println!("============================================================================================");
                return url_list;
            },
        }
    }
}

async fn tor_connect() {
    let tor_path = std::fs::read_to_string("./tor_path.txt").expect("Err");
    let mut tor_output = Command::new(tor_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("ERR");
    let stdout_reader = BufReader::new(tor_output.stdout.take().expect("Failed to open stdout"));
    std::thread::spawn(move || {
        for line in stdout_reader.lines() {
            if let Ok(line) = line {
                println!("Tor: {}", line);
            }
        }
    });
    sleep(Duration::from_millis(10000));
    get_request().await;
    let status = tor_output.wait();
    println!("{}",status.unwrap())
}

#[tokio::main]
async fn main() {
    tor_connect().await;
}

