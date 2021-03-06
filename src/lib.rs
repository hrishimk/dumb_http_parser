extern crate url;

use std::collections::HashMap;
use std::str;

#[derive(Debug)]
pub struct HttpParser<'a> {
    buf: &'a [u8],
    method: HttpMethod,
    src: [usize; 2],
    cookie: [usize; 2],
    content_length: [usize; 2],
    content_type: ContentType,
    body: [usize; 2],
}

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
    UNKNOWN,
}

#[derive(Debug)]
struct ContentType {
    media_type: [usize; 2],
    charset: [usize; 2],
    boundary: [usize; 2],
}

impl ContentType {
    fn empty() -> Self {
        Self {
            media_type: [0, 0],
            charset: [0, 0],
            boundary: [0, 0],
        }
    }
}

impl<'a> HttpParser<'a> {
    pub fn new(buf: &'a [u8]) -> HttpParser<'a> {
        //println!("{}", s);
        HttpParser {
            buf: buf,
            method: HttpMethod::UNKNOWN,
            src: [0, 0],
            cookie: [0, 0],
            content_length: [0, 0],
            content_type: ContentType::empty(),
            body: [0, 0],
        }
    }

    pub fn parse(&mut self) {
        let mut cur_line_start = false;
        let mut cur_line = 0;
        let mut cur_line_start_pos = 0;
        //let mut cur_line_key_set = false;
        let mut cur_line_first_space_set = false;
        let mut cur_line_first_space_pos = 0;
        let mut cur_line_last_was_r = false;

        let mut cur_line_last_header_key: [usize; 2] = [0, 0];

        let biterator = self.buf.iter();

        for (i, n) in biterator.enumerate() {
            if cur_line_start {
                if *n == b'\r' {
                    let len = self.buf.len();
                    if i + 2 > len - 1 {
                        self.set_body([0, 0]);
                    } else {
                        self.set_body([i + 2, len]);
                    }
                    break;
                }
            }

            if cur_line_last_was_r {
                cur_line_start = true;
                cur_line += 1;
                cur_line_start_pos = i + 1;
                //cur_line_key_set = false;
                cur_line_first_space_set = false;
                cur_line_first_space_pos = i;
                cur_line_last_was_r = false;
                cur_line_last_header_key = [0, 0];
                continue;
            } else {
                cur_line_start = false;
            }

            if cur_line == 0 {
                if *n == b' ' {
                    if cur_line_first_space_set {
                        self.set_src([cur_line_first_space_pos + 1, i]);
                    } else {
                        match &self.buf[0..i] {
                            b"GET" => self.set_method(HttpMethod::GET),
                            b"POST" => self.set_method(HttpMethod::POST),
                            _ => self.set_method(HttpMethod::UNKNOWN),
                        }

                        //self.set_method([0, i]);
                        cur_line_first_space_set = true;
                        cur_line_first_space_pos = i;
                    }
                }

                if *n == b'\r' {
                    cur_line_last_was_r = true;
                }
            } else {
                if *n == b' ' && !cur_line_first_space_set {
                    cur_line_first_space_pos = i;
                    cur_line_first_space_set = true;
                    cur_line_last_header_key = [cur_line_start_pos, i - 1];
                } else {
                    if *n == b'\r' {
                        let key = str::from_utf8(
                            &self.buf[cur_line_last_header_key[0]..cur_line_last_header_key[1]],
                        );

                        let key = match key {
                            Ok(x) => x.to_lowercase(),
                            Err(_) => "".to_string(),
                        };

                        let key = key.as_str();
                        if "cookie" == key {
                            self.set_cookie([cur_line_first_space_pos + 1, i]);
                        } else if "content-length" == key {
                            self.set_content_length([cur_line_first_space_pos + 1, i]);
                        } else if "content-type" == key {
                            self.set_content_type([cur_line_first_space_pos + 1, i]);
                        }

                        cur_line_last_was_r = true;
                    }
                }
            }
        }
    }

    pub fn set_method(&mut self, method: HttpMethod) {
        self.method = method;
    }

    pub fn set_src(&mut self, method: [usize; 2]) {
        self.src = method;
    }

    pub fn set_cookie(&mut self, method: [usize; 2]) {
        self.cookie = method;
    }

    pub fn set_content_length(&mut self, method: [usize; 2]) {
        self.content_length = method;
    }

    fn split(&mut self, data: [usize; 2], splitter: u8) -> Vec<[usize; 2]> {
        let mut points: Vec<usize> = vec![];
        let mut chunks: Vec<[usize; 2]> = vec![];

        for i in data[0]..data[1] {
            let d = self.buf[i];
            if d == splitter {
                points.push(i);
            }
        }

        let mut start = data[0];
        let end = data[1];
        for point in points {
            chunks.push([start, point]);
            start = point + 1;
        }

        chunks.push([start, end]);

        chunks
    }

    pub fn set_content_type(&mut self, method: [usize; 2]) {
        let items = self.split(method, b';');

        for (i, item) in items.iter().enumerate() {
            if i == 0 {
                self.content_type.media_type = *item;
                continue;
            }

            let splitted = self.split([item[0], item[1]], b'=');
            if splitted.len() < 2 {
                continue;
            }

            let key = str::from_utf8(&self.buf[item[0]..splitted[0][1]]);

            let key = match key {
                Ok(x) => x.trim().to_lowercase(),
                Err(_) => "".to_string(),
            };

            if key == "charset" {
                self.content_type.charset = [splitted[1][0], item[1]];
            } else if key == "boundary" {
                self.content_type.boundary = [splitted[1][0], item[1]];
            }
        }
    }

    pub fn set_body(&mut self, method: [usize; 2]) {
        self.body = method;
    }

    pub fn get_method(&self) -> &HttpMethod {
        &self.method
    }

    pub fn get_content_length(&self) -> usize {
        str::from_utf8(&self.buf[self.content_length[0]..self.content_length[1]])
            .unwrap_or("0")
            .parse::<usize>()
            .unwrap_or(0)
    }

    pub fn get_content_type(&self) -> &str {
        str::from_utf8(&self.buf[self.content_type.media_type[0]..self.content_type.media_type[1]])
            .unwrap()
    }

    pub fn get_charset(&self) -> &str {
        str::from_utf8(&self.buf[self.content_type.charset[0]..self.content_type.charset[1]])
            .unwrap()
    }

    pub fn get_multipart_boundary(&self) -> &str {
        str::from_utf8(&self.buf[self.content_type.boundary[0]..self.content_type.boundary[1]])
            .unwrap()
    }

    pub fn get_src(&self) -> &str {
        str::from_utf8(&self.buf[self.src[0]..self.src[1]]).unwrap()
    }

    pub fn get_cookie(&self) -> &str {
        str::from_utf8(&self.buf[self.cookie[0]..self.cookie[1]]).unwrap()
    }

    pub fn get_body(&self) -> &str {
        str::from_utf8(&self.buf[self.body[0]..self.body[1]]).unwrap()
    }

    pub fn get_page(&self) -> &str {
        let src = &self.buf[self.src[0]..self.src[1]];

        for (i, n) in src.iter().enumerate() {
            if *n == b'?' {
                return str::from_utf8(&src[..i]).unwrap();
            }
        }

        str::from_utf8(&self.buf[self.src[0]..self.src[1]]).unwrap()
    }

    pub fn get_params(&self) -> &str {
        let src = &self.buf[self.src[0]..self.src[1]];

        for (i, n) in src.iter().enumerate().rev() {
            if *n == b'?' {
                return str::from_utf8(&src[i + 1..]).unwrap();
            }
        }

        str::from_utf8(&self.buf[self.src[0]..self.src[1]]).unwrap()
    }

    pub fn get_params_index(&self) -> (usize, usize) {
        let src = &self.buf[self.src[0]..self.src[1]];

        for (i, n) in src.iter().enumerate().rev() {
            if *n == b'?' {
                return (self.src[0] + i + 1, self.src[1]);
            }
        }
        (0, 0)
    }

    pub fn get_map(&self, nature: &str, splitter: &str) -> HashMap<&str, &str> {
        let mut params: HashMap<&str, &str> = HashMap::new();
        let params_str;

        if nature == "params" {
            params_str = self.get_params().split(splitter);
        } else if nature == "cookies" {
            params_str = self.get_cookie().split(splitter);
        } else {
            params_str = self.get_body().split(splitter);
        }
        for pair in params_str {
            let key_val: Vec<&str> = pair.split('=').collect();

            let key = key_val.get(0).unwrap_or(&"");
            let val = key_val.get(1).unwrap_or(&"");

            params.insert(key, val);
        }

        params
    }

    pub fn get_post_params<'b>(&self) -> HashMap<String, String>
    where
        'a: 'b,
    {
        let ctype = self.get_content_type();
        if ctype.trim().to_lowercase() == "application/x-www-form-urlencoded" {
            return self.get_post_params_url_encoded();
        }

        self.get_body_map()
    }

    pub fn get_post_params_url_encoded(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();

        let body = &self.buf[self.body[0]..self.body[1]];
        let a = url::form_urlencoded::parse(body);
        for (key, value) in a.into_owned() {
            params.insert(key, value);
        }

        params
    }

    pub fn get_params_map(&self) -> HashMap<String, String> {
        let (i1, i2) = self.get_params_index();
        let a = url::form_urlencoded::parse(&self.buf[i1..i2]);

        let mut map: HashMap<String, String> = HashMap::new();

        for (key, value) in a.into_owned() {
            map.insert(key, value);
        }
        map
    }

    pub fn get_body_map(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();

        let params_str = self.get_body().split('&');

        for pair in params_str {
            let key_val: Vec<&str> = pair.split('=').collect();

            let key = key_val.get(0).unwrap_or(&"").to_string();
            let val = key_val.get(1).unwrap_or(&"").to_string();

            params.insert(key, val);
        }
        params
    }

    pub fn get_cookie_map(&self) -> HashMap<&str, &str> {
        self.get_map("cookies", "; ")
    }

    pub fn is_body_read(&self) -> bool {
        if (self.body[1] - self.body[0]) < self.get_content_length() {
            false
        } else {
            true
        }
    }

    pub fn get_body_remain(&self) -> usize {
        self.get_content_length() - self.body[1]
    }

    pub fn check_data(buf: &[u8]) -> usize {
        let iter = buf.iter();

        let mut cur_line_start = false;
        let mut cur_line = 0;
        let mut cur_line_start_pos = 0;
        //let mut cur_line_key_set = false;
        let mut cur_line_first_space_set = false;
        let mut cur_line_first_space_pos = 0;
        let mut cur_line_last_was_r = false;

        let mut cur_line_last_header_key: [usize; 2] = [0, 0];

        let mut content_len = 0;
        let mut body_len = 0;
        //let mut body_begin = 0;

        for (i, n) in iter.enumerate() {
            if cur_line_start {
                if *n == b'\r' {
                    let len = buf.len();
                    if i + 2 > len - 1 {
                        body_len = 0;
                    } else {
                        body_len = len - i - 2;
                    }
                    break;
                }
            }

            if cur_line_last_was_r {
                cur_line_start = true;
                cur_line += 1;
                cur_line_start_pos = i + 1;
                //cur_line_key_set = false;
                cur_line_first_space_set = false;
                cur_line_first_space_pos = i;
                cur_line_last_was_r = false;
                cur_line_last_header_key = [0, 0];
                continue;
            } else {
                cur_line_start = false;
            }

            if cur_line != 0 {
                if *n == b' ' && !cur_line_first_space_set {
                    cur_line_first_space_pos = i;
                    cur_line_first_space_set = true;
                    cur_line_last_header_key = [cur_line_start_pos, i - 1];
                } else {
                    if *n == b'\r' {
                        let key = str::from_utf8(
                            &buf[cur_line_last_header_key[0]..cur_line_last_header_key[1]],
                        );

                        let key = match key {
                            Ok(x) => x.to_lowercase(),
                            Err(_) => "".to_string(),
                        };

                        let key = key.as_str();

                        if "content-length" == key {
                            content_len = str::from_utf8(&buf[cur_line_first_space_pos + 1..i])
                                .unwrap_or("0")
                                .parse::<usize>()
                                .unwrap_or(0);
                        }

                        cur_line_last_was_r = true;
                    }
                }
            } else {
                //Check first line
                if *n == b'\r' {
                    cur_line_last_was_r = true;
                }
            }
        }

        content_len - body_len
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn get_str_params_works() {
        let a = b"GET /get?type=dbs&active=1 HTTP/1.1\r\n\r\n";
        let mut parser = HttpParser::new(a);
        parser.parse();
        let gparams = parser.get_params();

        assert_eq!("type=dbs&active=1", gparams);
    }

    #[test]
    fn get_str_body_works() {
        let a = b"GET /get?type=dbs&active=1 HTTP/1.1\r\n\r\nHelloIamTheBody";
        let mut parser = HttpParser::new(a);
        parser.parse();
        let gparams = parser.get_body();

        assert_eq!("HelloIamTheBody", gparams);
    }

    #[test]
    fn params_to_hashmap_works() {
        let a = b"GET /get?type=dbs&active=1 HTTP/1.1\r\n\r\n";
        let mut parser = HttpParser::new(a);
        parser.parse();
        let mut test_map = HashMap::new();
        test_map.insert("type".to_string(), "dbs".to_string());
        test_map.insert("active".to_string(), "1".to_string());
        assert_eq!(test_map, parser.get_params_map());
    }

    #[test]
    fn get_content_length_works() {
        let a = b"GET /get?type=dbs&active=1 HTTP/1.1\r\nContent-Length: 100\r\n\r\n";
        let mut parser = HttpParser::new(a);
        parser.parse();

        assert_eq!(100, parser.get_content_length());
    }
    #[test]
    fn check_data_checker_half() {
        let a = b"GET /get?type=dbs&active=1 HTTP/1.1\r\nContent-Length: 8\r\n\r\nabcdef";

        assert_ne!(HttpParser::check_data(a), 0);
    }

    #[test]
    fn check_data_checker_full() {
        let a = b"GET /get?type=dbs&active=1 HTTP/1.1\r\nContent-Length: 6\r\n\r\nabcdef";

        assert_eq!(HttpParser::check_data(a), 0);
    }

    #[test]
    fn read_content_type() {
        let a = b"POST /get?type=dbs&active=1 HTTP/1.1\r\nContent-Length: 6\r\nContent-Type: text/html; charset=utf-8\r\n\r\nabcdef";
        let mut parser = HttpParser::new(a);
        parser.parse();

        assert_eq!(parser.get_content_type(), "text/html");
    }

    #[test]
    fn read_charset() {
        let a = b"POST /get?type=dbs&active=1 HTTP/1.1\r\nContent-Length: 6\r\nContent-Type: text/html; charset=utf-8\r\n\r\nabcdef";
        let mut parser = HttpParser::new(a);
        parser.parse();
        assert_eq!(parser.get_charset(), "utf-8");
    }

    #[test]
    fn read_charset_multiple() {
        let a = b"POST /get?type=dbs&active=1 HTTP/1.1\r\nContent-Type: text/html; charset=utf-8; boundary=something\r\nContent-Length: 6\r\n\r\nabcdef";
        let mut parser = HttpParser::new(a);
        parser.parse();
        assert_eq!(parser.get_charset(), "utf-8");
    }

    #[test]
    fn read_multipart_boundary() {
        let a = b"POST /get?type=dbs&active=1 HTTP/1.1\r\nContent-Length: 6\r\nContent-Type: text/html; charset=utf-8; boundary=something\r\n\r\nabcdef";
        let mut parser = HttpParser::new(a);
        parser.parse();
        assert_eq!(parser.get_multipart_boundary(), "something");
    }

    #[test]
    fn post_url_encoded() {
        let a = b"POST /get?type=dbs&active=1 HTTP/1.1\r\nContent-Length: 6\r\nContent-Type: application/x-www-form-urlencoded; charset=utf-8; boundary=something\r\n\r\nkey1=20%25&key2=10+%2B+10&key3=data3";

        let mut parser = HttpParser::new(a);
        parser.parse();

        let mut params = HashMap::new();

        params.insert("key1".to_string(), "20%".to_string());
        params.insert("key2".to_string(), "10 + 10".to_string());
        params.insert("key3".to_string(), "data3".to_string());

        let sparams = parser.get_post_params();

        println!("{:#?}", sparams);

        assert_eq!(params, sparams);
    }

    #[test]
    fn post_url_encoded_without_header() {
        let a = b"POST /get?type=dbs&active=1 HTTP/1.1\r\nContent-Length: 6\r\nContent-Type: text/html; charset=utf-8; boundary=something\r\n\r\nkey1=20%&key2=10 + 10&key3=data3";
        let mut parser = HttpParser::new(a);
        parser.parse();

        let mut params = HashMap::new();

        params.insert("key1".to_string(), "20%".to_string());
        params.insert("key2".to_string(), "10 + 10".to_string());
        params.insert("key3".to_string(), "data3".to_string());

        let sparams = parser.get_post_params();

        println!("{:#?}", sparams);

        assert_eq!(params, sparams);
    }

}
