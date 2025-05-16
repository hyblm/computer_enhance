use crate::{profile::DropTimer, time_function, Pair};

pub fn parse_haversine_pairs(json_slice: &str) -> Result<Vec<Pair>, String> {
    time_function!(4);

    let mut slice = json_slice;

    slice = whitespace(slice);
    slice = consume(slice, "{")?;
    slice = whitespace(slice);

    slice = consume(slice, "\"pairs\"")?;
    slice = whitespace(slice);
    slice = consume(slice, ":")?;
    slice = whitespace(slice);

    let mut pairs = Vec::new();
    slice = array(slice, &mut pairs)?;
    slice = whitespace(slice);
    _ = consume(slice, "}")?;

    Ok(pairs)
}

fn array<'a>(mut slice: &'a str, pairs: &mut Vec<Pair>) -> Result<&'a str, String> {
    slice = consume(slice, "[")?;

    loop {
        slice = whitespace(slice);
        let (rest, pair) = pair(slice)?;
        slice = whitespace(rest);

        pairs.push(pair);

        match consume(slice, ",") {
            Ok(rest) => slice = rest,
            Err(_) => break,
        }
    }
    slice = whitespace(slice);
    slice = consume(slice, "]")?;

    Ok(slice)
}

fn pair(mut slice: &str) -> Result<(&str, Pair), String> {
    let x0;
    let y0;
    let x1;
    let y1;
    slice = consume(slice, "{")?;
    slice = whitespace(slice);

    slice = consume(slice, "\"x0\"")?;
    slice = whitespace(slice);
    slice = consume(slice, ":")?;
    slice = whitespace(slice);
    (slice, x0) = number(slice)?;
    slice = whitespace(slice);
    slice = consume(slice, ",")?;
    slice = whitespace(slice);

    slice = consume(slice, "\"y0\"")?;
    slice = whitespace(slice);
    slice = consume(slice, ":")?;
    slice = whitespace(slice);
    (slice, y0) = number(slice)?;
    slice = whitespace(slice);
    slice = consume(slice, ",")?;
    slice = whitespace(slice);

    slice = consume(slice, "\"x1\"")?;
    slice = whitespace(slice);
    slice = consume(slice, ":")?;
    slice = whitespace(slice);
    (slice, x1) = number(slice)?;
    slice = whitespace(slice);
    slice = consume(slice, ",")?;
    slice = whitespace(slice);

    slice = consume(slice, "\"y1\"")?;
    slice = whitespace(slice);
    slice = consume(slice, ":")?;
    slice = whitespace(slice);
    (slice, y1) = number(slice)?;

    slice = whitespace(slice);
    slice = consume(slice, "}")?;

    let pair = Pair { x0, y0, x1, y1 };

    Ok((slice, pair))
}

fn number(mut slice: &str) -> Result<(&str, f64), String> {
    time_function!(5);
    let sign = match consume(slice, "-") {
        Ok(rest) => {
            slice = rest;
            -1.
        }
        Err(_) => 1.,
    };

    let Some(len) = slice.find(|x: char| !(x.is_ascii_digit() || x == '.')) else {
        return Err("Number never ends".to_owned());
    };

    let (float, rest) = slice.split_at(len);
    slice = rest;
    let Ok(float) = float.parse::<f64>() else {
        return Err(format!("Failed to parse number: {float}"));
    };

    Ok((slice, sign * float))
}

fn consume<'a>(slice: &'a str, content: &'static str) -> Result<&'a str, String> {
    if !content.chars().zip(slice.chars()).all(|(c, b)| c == b) {
        return Err(format!("Failed to consume \"{content}\""));
    }

    Ok(&slice[content.len()..])
}

fn whitespace(slice: &str) -> &str {
    let i = slice.find(|x: char| !x.is_whitespace()).unwrap_or_default();
    let (_, slice) = slice.split_at(i);
    slice
}
