use crate::query_builder::args::QueryBuilderArgs;
use crate::Result;
use sqlx::query::QueryAs;

pub fn replace_placeholders<T: Iterator<Item = String>>(
    sql: &str,
    placeholder_generator: &mut T,
) -> Result<(String, usize)> {
    let mut placeholder_count = 0usize;
    let mut buf = String::with_capacity(sql.len() + 16);
    let bytes = sql.as_bytes();
    let mut cursor = 0;
    let mut copied_until = 0;

    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\'' => cursor = skip_single_quoted_string(bytes, cursor),
            b'"' => cursor = skip_quoted_identifier(bytes, cursor, b'"'),
            b'`' => cursor = skip_quoted_identifier(bytes, cursor, b'`'),
            b'[' => cursor = skip_bracket_quoted_identifier(bytes, cursor),
            b'-' if bytes.get(cursor + 1) == Some(&b'-') => cursor = skip_line_comment(bytes, cursor),
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => cursor = skip_block_comment(bytes, cursor),
            b'$' => {
                if let Some(next_cursor) = skip_dollar_quoted_string(sql, cursor) {
                    cursor = next_cursor;
                } else if let Some((n, next_cursor)) = parse_numbered_placeholder(bytes, cursor) {
                    placeholder_count = std::cmp::max(placeholder_count, n);
                    cursor = next_cursor;
                } else {
                    cursor += 1;
                }
            }
            b'?' => {
                buf.push_str(&sql[copied_until..cursor]);
                buf.push_str(&placeholder_generator.next().unwrap());
                placeholder_count += 1;
                cursor += 1;
                copied_until = cursor;
            }
            _ => cursor += 1,
        }
    }

    buf.push_str(&sql[copied_until..]);
    Ok((buf, placeholder_count))
}

fn skip_single_quoted_string(bytes: &[u8], mut cursor: usize) -> usize {
    cursor += 1;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\'' if bytes.get(cursor + 1) == Some(&b'\'') => cursor += 2,
            b'\'' => return cursor + 1,
            b'\\' if cursor + 1 < bytes.len() => cursor += 2,
            _ => cursor += 1,
        }
    }
    bytes.len()
}

fn skip_quoted_identifier(bytes: &[u8], mut cursor: usize, quote: u8) -> usize {
    cursor += 1;
    while cursor < bytes.len() {
        if bytes[cursor] == quote {
            if bytes.get(cursor + 1) == Some(&quote) {
                cursor += 2;
            } else {
                return cursor + 1;
            }
        } else {
            cursor += 1;
        }
    }
    bytes.len()
}

fn skip_bracket_quoted_identifier(bytes: &[u8], mut cursor: usize) -> usize {
    cursor += 1;
    while cursor < bytes.len() {
        if bytes[cursor] == b']' {
            if bytes.get(cursor + 1) == Some(&b']') {
                cursor += 2;
            } else {
                return cursor + 1;
            }
        } else {
            cursor += 1;
        }
    }
    bytes.len()
}

fn skip_line_comment(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() && bytes[cursor] != b'\n' {
        cursor += 1;
    }
    cursor
}

fn skip_block_comment(bytes: &[u8], mut cursor: usize) -> usize {
    cursor += 2;
    let mut depth = 1usize;
    while cursor + 1 < bytes.len() {
        match (bytes[cursor], bytes[cursor + 1]) {
            (b'/', b'*') => {
                depth += 1;
                cursor += 2;
            }
            (b'*', b'/') => {
                depth -= 1;
                cursor += 2;
                if depth == 0 {
                    return cursor;
                }
            }
            _ => cursor += 1,
        }
    }
    bytes.len()
}

fn parse_numbered_placeholder(bytes: &[u8], cursor: usize) -> Option<(usize, usize)> {
    let mut end = cursor + 1;
    if !bytes.get(end).is_some_and(u8::is_ascii_digit) {
        return None;
    }
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    let number = std::str::from_utf8(&bytes[cursor + 1..end]).ok()?.parse().ok()?;
    Some((number, end))
}

fn skip_dollar_quoted_string(sql: &str, cursor: usize) -> Option<usize> {
    let (delimiter, content_start) = dollar_quote_delimiter(sql, cursor)?;
    let end = sql[content_start..]
        .find(delimiter)
        .map(|offset| content_start + offset + delimiter.len())
        .unwrap_or(sql.len());
    Some(end)
}

fn dollar_quote_delimiter(sql: &str, cursor: usize) -> Option<(&str, usize)> {
    let bytes = sql.as_bytes();
    if bytes.get(cursor) != Some(&b'$') {
        return None;
    }

    match bytes.get(cursor + 1) {
        Some(b'$') => Some((&sql[cursor..cursor + 2], cursor + 2)),
        Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => {
            let mut end = cursor + 2;
            while bytes.get(end).is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_') {
                end += 1;
            }
            (bytes.get(end) == Some(&b'$')).then_some((&sql[cursor..=end], end + 1))
        }
        _ => None,
    }
}

pub(super) fn query_as_with_recast_lifetime<'q, 'r, DB, Model>(
    s: &'q str,
    args: QueryBuilderArgs<'r, DB>,
) -> QueryAs<'q, DB, Model, QueryBuilderArgs<'q, DB>>
where
    'r: 'q,
    DB: sqlx::Database,
    Model: for<'s> sqlx::FromRow<'s, DB::Row>,
{
    // unsafe is safe b/c 'r: 'q. Rust isn't smart enough to know that downcasting of traits is safe, because when traits get lifetimes, it doesn't
    // know if the lifetime is covariant or contravariant, so it enforces equivalence. See: https://www.reddit.com/r/rust/comments/rox4j9/lifetime_inference_fails_when_lifetime_is_part_of/
    // But we know the trait is implemented by a struct, not a function, so we can do the downcast safely. Yay!
    let recast_args = unsafe { std::mem::transmute::<_, QueryBuilderArgs<'q, DB>>(args) };
    sqlx::query_as_with(s, recast_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Result;

    fn dollar_placeholders() -> impl Iterator<Item = String> {
        (1..).map(|i| format!("${i}"))
    }

    fn assert_rewrite(input: &str, expected_sql: &str, expected_placeholder_count: usize) -> Result<()> {
        let mut placeholder_generator = dollar_placeholders();
        let (sql, placeholder_count) = replace_placeholders(input, &mut placeholder_generator)?;
        assert_eq!(sql, expected_sql);
        assert_eq!(placeholder_count, expected_placeholder_count);
        Ok(())
    }

    #[test]
    fn test_replace_placeholders() -> Result<()> {
        let mut placeholder_generator = dollar_placeholders();
        let (sql, placeholder_count) = replace_placeholders(
            "SELECT * FROM users WHERE id = ? OR id = ? OR id = ?",
            &mut placeholder_generator,
        )?;
        assert_eq!(sql, "SELECT * FROM users WHERE id = $1 OR id = $2 OR id = $3");
        assert_eq!(placeholder_count, 3);
        Ok(())
    }

    #[test]
    fn test_leave_placeholders_alone() -> Result<()> {
        let mut placeholder_generator = dollar_placeholders();
        let (sql, placeholder_count) =
            replace_placeholders("SELECT * FROM users WHERE email = $1", &mut placeholder_generator)?;
        assert_eq!(sql, "SELECT * FROM users WHERE email = $1");
        assert_eq!(placeholder_count, 1);
        Ok(())
    }

    #[test]
    fn test_counts_highest_numbered_placeholder() -> Result<()> {
        let mut placeholder_generator = dollar_placeholders();
        let (sql, placeholder_count) =
            replace_placeholders("SELECT * FROM users WHERE email = $2", &mut placeholder_generator)?;
        assert_eq!(sql, "SELECT * FROM users WHERE email = $2");
        assert_eq!(placeholder_count, 2);
        Ok(())
    }

    #[test]
    fn test_ignores_question_marks_inside_strings() -> Result<()> {
        assert_rewrite("SELECT '?'", "SELECT '?'", 0)?;
        assert_rewrite("SELECT '$1'", "SELECT '$1'", 0)?;
        assert_rewrite(
            "SELECT '?', 'it''s?', 'backslash\\'?' WHERE id = ?",
            "SELECT '?', 'it''s?', 'backslash\\'?' WHERE id = $1",
            1,
        )?;
        assert_rewrite(
            r#"SELECT E'escaped \' ? $4' WHERE id = ?"#,
            r#"SELECT E'escaped \' ? $4' WHERE id = $1"#,
            1,
        )?;
        Ok(())
    }

    #[test]
    fn test_ignores_question_marks_inside_quoted_identifiers() -> Result<()> {
        assert_rewrite(
            r#"SELECT "weird?column", `other?column`, [third?column] FROM users WHERE id = ?"#,
            r#"SELECT "weird?column", `other?column`, [third?column] FROM users WHERE id = $1"#,
            1,
        )?;
        assert_rewrite(
            r#"SELECT "col$9", `col$8`, [col$7] FROM users WHERE id = ?"#,
            r#"SELECT "col$9", `col$8`, [col$7] FROM users WHERE id = $1"#,
            1,
        )?;
        Ok(())
    }

    #[test]
    fn test_ignores_question_marks_inside_comments() -> Result<()> {
        assert_rewrite(
            "SELECT * FROM users -- ?\nWHERE id = ? /* ? */ AND name = ?",
            "SELECT * FROM users -- ?\nWHERE id = $1 /* ? */ AND name = $2",
            2,
        )?;
        assert_rewrite(
            "SELECT * FROM users -- ? $8\nWHERE id = ? /* ? $9 */",
            "SELECT * FROM users -- ? $8\nWHERE id = $1 /* ? $9 */",
            1,
        )?;
        assert_rewrite(
            "SELECT * FROM users /* outer ? $7 /* inner ? $8 */ end */ WHERE id = ?",
            "SELECT * FROM users /* outer ? $7 /* inner ? $8 */ end */ WHERE id = $1",
            1,
        )?;
        Ok(())
    }

    #[test]
    fn test_ignores_placeholders_inside_dollar_quoted_strings() -> Result<()> {
        assert_rewrite(
            "SELECT $$?$1$$, $tag$?$2$tag$ FROM users WHERE id = ?",
            "SELECT $$?$1$$, $tag$?$2$tag$ FROM users WHERE id = $1",
            1,
        )?;
        assert_rewrite(
            "SELECT $body$SELECT ? WHERE id = $3$body$ WHERE id = ?",
            "SELECT $body$SELECT ? WHERE id = $3$body$ WHERE id = $1",
            1,
        )?;
        Ok(())
    }

    #[test]
    fn test_rewrites_real_placeholders_around_literal_sections_in_order() -> Result<()> {
        assert_rewrite(
            "SELECT * FROM users WHERE first = ? AND note = '?' AND second = ?",
            "SELECT * FROM users WHERE first = $1 AND note = '?' AND second = $2",
            2,
        )?;
        assert_rewrite(
            "SELECT * FROM users WHERE first = ? /* ? */ AND second = ? -- ?\nAND third = ?",
            "SELECT * FROM users WHERE first = $1 /* ? */ AND second = $2 -- ?\nAND third = $3",
            3,
        )?;
        Ok(())
    }

    #[test]
    fn test_handles_unterminated_quoted_sections_as_literal_sql() -> Result<()> {
        let mut placeholder_generator = dollar_placeholders();
        let (sql, placeholder_count) = replace_placeholders("SELECT '? WHERE id = ?", &mut placeholder_generator)?;
        assert_eq!(sql, "SELECT '? WHERE id = ?");
        assert_eq!(placeholder_count, 0);
        Ok(())
    }
}
