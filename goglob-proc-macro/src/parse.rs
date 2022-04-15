use proc_macro2::{Delimiter, Literal, Span, TokenStream, TokenTree};
use std::char;

macro_rules! unexpected_content {
    () => {
        "expected string literal"
    };
}

pub(crate) struct ParseError(pub(crate) Span, pub(crate) &'static str);

pub(crate) fn parse_input(mut input: TokenStream) -> Result<(String, Span), ParseError> {
    loop {
        let mut tokens = input.into_iter();
        let token = match tokens.next() {
            Some(token) => token,
            None => {
                return Err(ParseError(
                    Span::call_site(),
                    concat!("unexpected end of input, ", unexpected_content!()),
                ))
            }
        };
        let span = token.span();
        let result = match token {
            // Unwrap any empty group which may be created from macro expansion.
            TokenTree::Group(group) if group.delimiter() == Delimiter::None => Err(group),
            TokenTree::Literal(literal) => match parse_literal(literal) {
                Ok(result) => Ok(result),
                Err(msg) => return Err(ParseError(span, msg)),
            },
            _ => return Err(ParseError(span, unexpected_content!())),
        };
        if let Some(token) = tokens.next() {
            return Err(ParseError(token.span(), "unexpected token"));
        }
        match result {
            Ok(result) => return Ok((result, span)),
            Err(group) =>
            // input is wrapped in a group, unwrap and continue
            {
                input = group.stream()
            }
        }
    }
}

fn parse_literal(literal: Literal) -> Result<String, &'static str> {
    let s = literal.to_string();
    let mut s_iter = s.char_indices();
    let (_, s0) = s_iter.next().ok_or(unexpected_content!())?;
    let (i, _) = s_iter.next().ok_or(unexpected_content!())?;
    match s0 {
        '"' => parse_cooked_content(&s[i..]),
        'r' => parse_raw_content(&s[i..]),
        _ => Err(unexpected_content!()),
    }
}

fn all_pounds(chars: &str) -> bool {
    chars.chars().all(|c| c == '#')
}

/// Parses raw string / bytes content after `r` prefix.
fn parse_raw_content(s: &str) -> Result<String, &'static str> {
    let q_start = s
        .find('"')
        .ok_or(concat!("missing '\"' after 'r', ", unexpected_content!()))?;
    let q_end = s
        .rfind('"')
        .ok_or(concat!("missing '\"' after 'r', ", unexpected_content!()))?;
    let end_pounds = {
        let e = &s[q_end..];
        let mut eci = e.char_indices();
        eci.next().ok_or("expected string literal")?;
        let (i, _) = e.char_indices().next().ok_or("expected string literal")?;
        &e[i..]
    };
    let start = {
        let mut result: Option<usize> = None;
        for (i, _) in s.char_indices() {
            if i > q_start {
                result = Some(i);
                break;
            }
        }
        result.ok_or(unexpected_content!())?
    };
    (all_pounds(&s[..q_start]) && all_pounds(end_pounds))
        .then(|| (&s[start..q_end]).to_string())
        .ok_or(unexpected_content!())
}

/// Parses the cooked string / bytes content within quotes.
fn parse_cooked_content(mut s: &str) -> Result<String, &'static str> {
    let end = s
        .rfind('"')
        .ok_or(concat!("missing '\"' after 'r', ", unexpected_content!()))?;
    s = &s[..end];
    let mut s_iter = s.chars().peekable();
    let mut result = String::new();
    while let Some(c) = s_iter.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        let c = s_iter
            .next()
            .ok_or(concat!("unexpected end-of-string, ", unexpected_content!()))?;
        match c {
            'x' => {
                let c = backslash_x(&mut s_iter)?;
                result.push(c);
            }
            'u' => {
                let c = backslash_u(&mut s_iter)?;
                result.push(c);
            }
            'n' => result.push('\n'),
            'r' => result.push('\r'),
            't' => result.push('\t'),
            '\\' => result.push('\\'),
            '0' => result.push('\0'),
            '\'' => result.push('\''),
            '"' => result.push('"'),
            '\r' | '\n' => {
                while let Some(&next) = s_iter.peek() {
                    if !next.is_whitespace() {
                        break;
                    }
                    s_iter.next();
                }
            }
            c => {
                return Err(Box::leak(
                    format!("unexpected char {:?} after \\", c).into_boxed_str(),
                ))
            }
        }
    }
    Ok(result)
}

fn backslash_x<I: Iterator<Item = char>>(i: &mut I) -> Result<char, &'static str> {
    let s0 = i.next().ok_or(unexpected_content!())?;
    let s1 = i.next().ok_or(unexpected_content!())?;
    let ch_b = hex_to_u8(s0)? * 0x10 + hex_to_u8(s1)?;
    Ok(char::from(ch_b))
}

fn hex_to_u8(c: char) -> Result<u8, &'static str> {
    let b = u8::try_from(c).map_err(|_| unexpected_content!())?;
    Ok(match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => {
            return Err(Box::leak(
                format!("unexpected non-hex character {:?} after \\x", b).into_boxed_str(),
            ))
        }
    })
}

fn backslash_u<I: Iterator<Item = char>>(i: &mut I) -> Result<char, &'static str> {
    (matches!(i.next(), Some('{'))).then(|| ()).ok_or(concat!(
        "unexpected unicode escape sequence, ",
        unexpected_content!()
    ))?;
    let mut hex_escape = String::new();
    loop {
        let c = i.next().ok_or(unexpected_content!())?;
        if c == '}' {
            break;
        }
        hex_escape.push(c);
    }
    let mut ch = 0;
    for c in hex_escape.chars() {
        ch *= 0x10;
        ch += u32::from(hex_to_u8(c)?);
    }
    char::from_u32(ch).ok_or_else(|| {
        let result: &'static mut str =
            Box::leak(format!("malformed character {:?}", ch).into_boxed_str());
        let result: &'static str = result;
        result
    })
}
