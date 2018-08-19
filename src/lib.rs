use std::collections::HashMap;
use std::str;

#[derive(Debug)]
pub struct HttpParser<'a> {
    buf: &'a [u8],
    method: HttpMethod,
    src: [usize; 2],
    cookie: [usize; 2],
    content_length: [usize; 2],
    content_type: [usize; 2],
}

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
    UNKNOWN,
}

impl<'a> HttpParser<'a> {
    pub fn new(buf: &'a [u8]) -> HttpParser<'a> {
        HttpParser {
            buf: buf,
            method: HttpMethod::UNKNOWN,
            src: [0, 0],
            cookie: [0, 0],
            content_length: [0, 0],
            content_type: [0, 0],
        }
    }

    pub fn parse(&mut self) {
        let mut cur_pos = 0;

        let mut cur_line = 0;
        let mut cur_line_start_pos = 0;
        let mut cur_line_key_set = false;
        let mut cur_line_first_space_set = false;
        let mut cur_line_first_space_pos = 0;
        let mut cur_line_last_was_r = false;

        let mut cur_line_last_header_key: [usize; 2] = [0, 0];

        let biterator = self.buf.iter();

        for (i, n) in biterator.enumerate() {
            cur_pos = i;

            if cur_line_last_was_r {
                cur_line += 1;
                cur_line_start_pos = i + 1;
                cur_line_key_set = false;
                cur_line_first_space_set = false;
                cur_line_first_space_pos = i;
                cur_line_last_was_r = false;
                cur_line_last_header_key = [0, 0];
                continue;
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
                    cur_line_last_header_key = [cur_line_start_pos, i - 2];
                } else {
                    if *n == b'\r' {
                        if b"Cookie:"
                            == &self.buf[cur_line_last_header_key[0]..cur_line_last_header_key[1]]
                        {
                            self.set_cookie([cur_line_first_space_pos, i - 1]);
                        } else if b"Content-Length:"
                            == &self.buf[cur_line_last_header_key[0]..cur_line_last_header_key[1]]
                        {
                            self.set_content_length([cur_line_first_space_pos, i - 1]);
                        } else if b"Content-Type:"
                            == &self.buf[cur_line_last_header_key[0]..cur_line_last_header_key[1]]
                        {
                            self.set_content_type([cur_line_first_space_pos, i - 1]);
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

    pub fn set_content_type(&mut self, method: [usize; 2]) {
        self.content_type = method;
    }

    pub fn get_method(&self) -> &HttpMethod {
        &self.method
    }

    pub fn get_src(&self) -> &str {
        str::from_utf8(&self.buf[self.src[0]..self.src[1]]).unwrap()
    }

    pub fn get_cookie(&self) -> &str {
        str::from_utf8(&self.buf[self.cookie[0]..self.cookie[1]]).unwrap()
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

    pub fn get_params_map(&self) -> HashMap<&str, &str> {
        let src = &self.buf[self.src[0]..self.src[1]];

        let mut params: HashMap<&str, &str> = HashMap::new();

        println!("src is {:#?}", src);

        let params_str = self.get_params().split('&');

        println!("params_str is {:?}", params_str);

        for pair in params_str {
            println!("\n\n{:#?}\n\n", pair);

            let key_val: Vec<&str> = pair.split('=').collect();

            let key = key_val.get(0).unwrap_or(&"");
            let val = key_val.get(1).unwrap_or(&"");

            params.insert(key, val);
        }

        params
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

        println!("method is {:?}", parser.get_method());

        assert_eq!("type=dbs&active=1", gparams);
    }

    #[test]
    fn params_to_hashmap_works() {
        let a = b"GET /get?type=dbs&active=1 HTTP/1.1\r\n\r\n";
        let mut parser = HttpParser::new(a);
        parser.parse();
        let mut test_map = HashMap::new();
        test_map.insert("type", "dbs");
        test_map.insert("active", "1");
        assert_eq!(test_map, parser.get_params_map());
    }
}
