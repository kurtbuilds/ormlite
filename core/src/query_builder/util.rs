use crate::query_builder::args::QueryBuilderArgs;
use crate::{CoreError, CoreResult};
use sqlparser::dialect::GenericDialect;
use sqlparser::tokenizer::{Token, Tokenizer};
use sqlx::query::QueryAs;

pub fn replace_placeholders<T: Iterator<Item = String>>(
    sql: &str,
    placeholder_generator: &mut T,
) -> CoreResult<(String, usize)> {
    let mut placeholder_count = 0usize;
    let dialect = GenericDialect {};
    // note this lib is inefficient because it's copying strings everywhere, instead
    // of using slices and an appropriate lifetime. probably want to swap out the lib at some point
    let tokens = Tokenizer::new(&dialect, sql).tokenize()?;
    // 16 is arbitrary here.
    let mut buf = String::with_capacity(sql.len() + 16);
    let mut it = tokens.iter();
    while let Some(tok) = it.next() {
        match tok {
            Token::Placeholder(_) => {
                buf.push_str(&placeholder_generator.next().unwrap());
                placeholder_count += 1;
            }
            Token::Char(c) => {
                match c {
                    '?' => {
                        buf.push_str(&placeholder_generator.next().unwrap());
                        placeholder_count += 1;
                    }
                    '$' => {
                        let next_tok = it.next();
                        if let Some(next_tok) = next_tok {
                            match next_tok {
                                Token::Number(text, _) => {
                                    let n = text.parse::<usize>().map_err(|_| CoreError::OrmliteError(
                                    format!("Failed to parse number after a $ during query tokenization. Value was: {text}"
                                    )))?;
                                    buf.push_str(&format!("${next_tok}"));
                                    placeholder_count = std::cmp::max(placeholder_count, n);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => buf.push(*c),
                }
            }
            _ => buf.push_str(&tok.to_string()),
        }
    }
    Ok((buf, placeholder_count))
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

    use crate::CoreResult;

    #[test]
    fn test_replace_placeholders() -> CoreResult<()> {
        let mut placeholder_generator = vec!["$1", "$2", "$3"].into_iter().map(|s| s.to_string());
        let (sql, placeholder_count) = replace_placeholders(
            "SELECT * FROM users WHERE id = ? OR id = ? OR id = ?",
            &mut placeholder_generator,
        )?;
        assert_eq!(sql, "SELECT * FROM users WHERE id = $1 OR id = $2 OR id = $3");
        assert_eq!(placeholder_count, 3);
        Ok(())
    }

    #[test]
    fn test_leave_placeholders_alone() -> CoreResult<()> {
        let mut placeholder_generator = vec!["$1", "$2", "$3"].into_iter().map(|s| s.to_string());
        let (sql, placeholder_count) =
            replace_placeholders("SELECT * FROM users WHERE email = $1", &mut placeholder_generator)?;
        assert_eq!(sql, "SELECT * FROM users WHERE email = $1");
        assert_eq!(placeholder_count, 1);
        Ok(())
    }
}
