use std::fmt::Display;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use diesel::RunQueryDsl;
use futures::stream::StreamExt;
use futures_core::future::BoxFuture;
use sea_orm::DatabaseBackend::Sqlite;
use sqlx::Row;
use ormlite::{Bindable, BuildsPartial, Db, Executable, Settable};
use sea_orm::entity::prelude::*;
use sea_orm::ConnectionTrait;


struct Task {
    name: &'static str,
    library: &'static str,
    orm: bool,
    db: &'static str,
    asyn: bool,
}

impl ToString for Task {
    fn to_string(&self) -> String {
        format!("{}:{},orm:{},async:{}",
                self.library, self.name,
                if self.orm { "yes" } else { "no" },
                if self.asyn { "yes" } else { "no" },
        )
    }
}


fn criterion_benchmark(c: &mut Criterion) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    // sqlx
    {
        let mut pool = rt.block_on(async move {
            let mut pool = sqlx::SqlitePool::connect("sqlite://:memory:").await.unwrap();
            sqlx::query(CREATE_TABLE_SQL).execute(&mut pool.acquire().await.unwrap()).await.unwrap();
            pool
        });
        c.bench_function(&Task {
            name: "insert-simple",
            library: "sqlx",
            orm: false,
            db: "sqlite",
            asyn: true,
        }.to_string(), |b| {
            b.to_async(&rt).iter(|| async {
                let x = sqlx::query_as::<_, Person>("INSERT INTO person (name, age) VALUES (?, ?) RETURNING *")
                    .bind("John Doe")
                    .bind(42u32)
                    .fetch_one(&mut pool.acquire().await.unwrap())
                    .await
                    .unwrap();
                black_box(&x);
            });
        });
    }

    {
        let mut pool = rt.block_on(async move {
            let mut pool = sqlx::SqlitePool::connect("sqlite://:memory:").await.unwrap();
            sqlx::query(CREATE_TABLE_SQL).execute(&mut pool.acquire().await.unwrap()).await.unwrap();
            pool
        });
        c.bench_function(&Task {
            name: "insert-simple",
            library: "ormlite",
            orm: true,
            db: "sqlite",
            asyn: true,
        }.to_string(), |b| {
            b.to_async(&rt).iter(|| async {
                let p = Person {
                    id: 0,
                    name: "John Doe".to_string(),
                    age: 42,
                }.insert(&mut pool.acquire().await.unwrap()).await.unwrap();
                black_box(&p);
            });
        });
    }

    {
        let mut pool = rt.block_on(async move {
            let mut pool = sqlx::SqlitePool::connect("sqlite://:memory:").await.unwrap();
            sqlx::query(CREATE_TABLE_SQL).execute(&mut pool.acquire().await.unwrap()).await.unwrap();
            pool
        });
        c.bench_function(&Task {
            name: "insert-simple",
            library: "ormlite/partial",
            orm: true,
            db: "sqlite",
            asyn: true,
        }.to_string(), |b| {
            b.to_async(&rt).iter(|| async {
                let p = Person::build()
                    .name("John Doe")
                    .age(42)
                    .insert(&mut pool.acquire().await.unwrap()).await.unwrap();
                black_box(&p);
            });
        });
    }

    {
        let t = Task {
            name: "insert-simple",
            library: "sea-orm",
            orm: true,
            db: "sqlite",
            asyn: true,
        };
        let mut pool = rt.block_on(async move {
            let pool: DatabaseConnection = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
            let exec_res: sea_orm::ExecResult = pool.execute(sea_orm::Statement::from_string(
                Sqlite,
                CREATE_TABLE_SQL.to_owned(),
            ))
                .await.unwrap();
            pool
        });
        c.bench_function(&t.to_string(), |b| b.to_async(&rt).iter(|| async {
            let p = ActiveModel {
                id: sea_orm::Unset(None),
                name: sea_orm::Set("John Doe".to_owned()),
                age: sea_orm::Set(42u8),
                ..Default::default() // no need to set primary key
            };
            let p: ActiveModel = p.insert(&pool).await.unwrap();
            black_box(&p);
        }));
    }

    {
        let t = Task {
            name: "insert-simple",
            library: "rusqlite",
            orm: false,
            db: "sqlite",
            asyn: false,
        };
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute(CREATE_TABLE_SQL, rusqlite::params![]).unwrap();
        c.bench_function(&t.to_string(), |b| b.iter(|| {
            let mut stmt = conn.prepare("INSERT INTO person (name, age) VALUES (?, ?) RETURNING *").unwrap();
            let mut person_iter = stmt.query_map(rusqlite::params!["John Doe", 42], |row| {
                Ok(Person {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    age: row.get(2)?,
                })
            }).unwrap();
            let p = person_iter.next().unwrap().unwrap();
            black_box(&p);
        }));
    }

    {
        let t = Task {
            name: "insert-simple",
            library: "diesel",
            orm: true,
            db: "sqlite",
            asyn: false,
        };
    }
    Ok(())
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);