
static TO_SKIP: [char; 3] = ['\n', '\t', ' '];

macro_rules! assert_length {
    ($input:expr) => {
        if $input.len() == 0 {
            return None
        }
    };
}

pub fn skip_stuff(input: &str) -> &str {
    for (i, char) in input.chars().enumerate()  {
        if !TO_SKIP.contains(&char) {
            return &input[i..]
        }
    }
    ""
}

pub fn get_any_char_s(input: &str) -> Option<(char, &str)> {
    let input = skip_stuff(input);
    assert_length!(input);
    if let Some(ch) = input.chars().nth(0) {
        return Some((ch, &input[1..]))
    }
    None
}

pub fn get_any_char(input: &str) -> Option<(char, &str)> {
    if let Some(ch) = input.chars().nth(0) {
        return Some((ch, &input[1..]))
    }
    None
}

pub fn get_char_s(input: &str, c: char) -> Option<&str> {
    let input = skip_stuff(input);
    assert_length!(input);
    if let Some(ch) = input.chars().nth(0) {
        if c == ch {
            return Some(&input[1..]);
        }
    }
    None
}

pub fn get_chars_s(input: &str, cs: Vec<char>) -> Option<(char, &str)> {
    let input = skip_stuff(input);
    assert_length!(input);
    if let Some(ch) = input.chars().nth(0) {
        if cs.contains(&ch) {
            return Some((ch, &input[1..]))
        }
    }
    None
}

pub fn get_number_string(input: &str) -> Option<(&str, &str)> {
    let input = skip_stuff(input);
    assert_length!(input);

    for (i, c) in input.chars().enumerate() {
        if i == 0 && !c.is_numeric() {
            return None
        }
        if !c.is_numeric() {
            return Some((&input[..i], &input[i..]))
        }
    }
    Some((input, ""))
}

pub fn get_number(input: &str) -> Option<(usize, &str)> {
    let n_res = get_number_string(input)?;
    if let Ok(number) = n_res.0.parse::<usize>() {
        Some((number, n_res.1))
    } else {
        None
    }
}

pub fn get_word<'a>(input: &'a str, word: &'a str) -> Option<&'a str> {
    let input = skip_stuff(input);
    assert_length!(input);
    if let Some(res) = skip_stuff(input).find(word) {
        if res == 0 {
            return Some(&input[word.len()..])
        }
    }
    None
}