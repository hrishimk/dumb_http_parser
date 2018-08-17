use std::str;

#[derive(Debug)]
pub struct HttpParser<'a> {
    buf: &'a [u8],
    method: [usize; 2],
    src: [usize; 2],
    cookie: [usize; 2],
}

impl<'a> HttpParser<'a> {
    pub fn new(buf: &'a [u8]) -> HttpParser<'a> {
        HttpParser {
            buf: buf,
            method: [0, 0],
            src: [0, 0],
            cookie: [0, 0],
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
                        self.set_method([0, i]);
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
                        }
                        cur_line_last_was_r = true;
                    }
                }
            }
        }
    }

    pub fn set_method(&mut self, method: [usize; 2]) {
        self.method = method;
    }

    pub fn set_src(&mut self, method: [usize; 2]) {
        self.src = method;
    }

    pub fn set_cookie(&mut self, method: [usize; 2]) {
        self.cookie = method;
    }

    pub fn get_method(&self) -> &str {
        str::from_utf8(&self.buf[self.method[0]..self.method[1]]).unwrap()
    }

    pub fn get_src(&self) -> &str {
        str::from_utf8(&self.buf[self.src[0]..self.src[1]]).unwrap()
    }

    pub fn get_cookie(&self) -> &str {
        str::from_utf8(&self.buf[self.cookie[0]..self.cookie[1]]).unwrap()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
