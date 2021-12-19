# Usage

```rust

// TODO figure out what exactly the derives are
#[derive(Partial, Insertable, FromRow)]
#[ormlite("user")]
struct User {
    id: Uuid,
    name: String,
    age: i32,

}

impl User {
    fn new(name: &str) -> Self {
        User { 
            id: Uuid::new_v4(),
            name: name.to_string(),
            age: 55,
        }
    }
}

fn main() {
    let mut user = User::new("Benjamin Button").insert(&db).await;
    user.age = 44;
    user.update_all_columns(&db).await;
    
    user.update_partial()
        .age(33)
        .update(&db).await;

    user.delete(&db);
    
    // ergonomic
    User::build() // a PartialUser
        .name("Marie Curie")
        .age(27)
        .insert(&db).await;
    
    // NB: Even on Postgres, this accepts ? parameters, because
    // the binding can become conditional.
    // I don't have a great alternative API idea to this. If you have other ideas let me know!
    User::select()
        .filter("id = ?").bind(user.id)
        // .join("foo", "foo.id = user.id and bar.created_at > ?", ())
        .join("join foo on foo.id = user.id and bar.created_at > ?").bind(foo)
        .filter()
        .having()
        .group_by()
        .order_by("name asc nulls last")
        .fetch_one(&db).await;
    
    User::query("select * from user where id = ?")
            .bind(user.id)
            .fetch_one(&db).await;

    // Possible future work. Not part of V1. If you want static checking, you can be 
    // creating the natural object (i.e. User)
    InsertUser {
      name: "new user's name".to_string()
    }.insert(&db);
    // select *
    // from foo on foo.id = bar.id  and bar > created_at > now() - interval '1 day'
}

```

- [ ] select with complex queries
  - ormx creates a new struct InsertUser. SeaORM creates a new struct ActiveModel.
  - why not call it PartialUser
- [ ] insert with default values.
- [ ] need something that you can bind a raw sql value (i.e. a string, but its NOT escaped.)



# Raw sqlx

```rust

fn main() {

    // Compile time checked SQL.
    struct Country { 
        country: String, 
        count: i64 
    }

    let countries = sqlx::query_as!(Country, "
SELECT country, COUNT(*) as count
FROM users
GROUP BY country
WHERE organization = $1
        ",
        organization
    )
        .fetch_all(&pool) // -> Vec<Country>
        .await?;

    // Runtime structured response.
    // 1. Can be put in type params...
    #[derive(sqlx::FromRow)]
    struct User { 
        name: String, 
        id: i64 
    }

    let mut stream = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 OR name = $2")
        .bind(user_email)
        .bind(user_name)
        .fetch(&mut conn);

    // 2. Or as a type annotation on return object.
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;
    
    // Response options.
    // .fetch_one() // 1 or Error
    // .fetch_optional() // -> 0 or 1
    // .fetch_many() // Vec<T>
    // .fetch() // impl Iter<Type=T>
    // .execute() // usize. number of affected rows.
    
    
}





```