# Migration (CLI)

In this section, we define the following simple schema with migrations.

![ER diagram of two entities, Bakery and Baker. Baker has a foreign key referencing Bakery.](./assets/er_diagram.png)

## Using `sea-orm-cli`

For beginners, it is recommended to use `sea-orm-cli` to define and run the migrations.

```sh
$ cargo install sea-orm-cli
$ sea-orm-cli migrate -h
```

Initialize the `migration` folder:

```sh
$ sea-orm-cli migrate init

# The folder structure will be as follows:

bakery-backend
│   ...  
│
└─── migration
│   │   Cargo.toml
│   │   README.md
│   │
│   └─── src
│       │   lib.rs
│       │   m20220101_000001_create_table.rs
│       │   main.rs
```

Update the migration files to define the `Bakery` and `Baker` tables:

```diff
- m20220101_000001_create_table.rs
+ m20220101_000001_create_bakery_table.rs
+ m20220101_000002_create_baker_table.rs
```

```rust, no_run
// m20220101_000001_create_bakery_table.rs
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_bakery_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Bakery table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Bakery::Table)
                    .col(
                        ColumnDef::new(Bakery::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Bakery::Name).string().not_null())
                    .col(ColumnDef::new(Bakery::ProfitMargin).double().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Bakery::Table).to_owned())
            .await
    }
}

// For ease of access
#[derive(Iden)]
pub enum Bakery {
    Table,
    Id,
    Name,
    ProfitMargin,
}
```

```rust, no_run
// m20220101_000002_create_baker_table.rs
use sea_orm_migration::prelude::*;

use super::m20220101_000001_create_bakery_table::Bakery;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20220101_000002_create_baker_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Baker table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Baker::Table)
                    .col(
                        ColumnDef::new(Baker::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Baker::Name).string().not_null())
                    .col(ColumnDef::new(Baker::ContactDetails).json())
                    .col(ColumnDef::new(Baker::BakeryId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-baker-bakery_id")
                            .from(Baker::Table, Baker::BakeryId)
                            .to(Bakery::Table, Bakery::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Baker table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Baker::Table).to_owned())
            .await
    }
}

// For ease of access
#[derive(Iden)]
pub enum Baker {
    Table,
    Id,
    Name,
    ContactDetails,
    BakeryId,
}
```

```rust, no_run
// migration/src/lib.rs
pub use sea_orm_migration::prelude::*;

// Add each migration file as a module
mod m20220101_000001_create_bakery_table;
mod m20220101_000002_create_baker_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // Define the order of migrations.
            Box::new(m20220101_000001_create_bakery_table::Migration),
            Box::new(m20220101_000002_create_baker_table::Migration),
        ]
    }
}
```

**Important**: Make sure the following features are enabled in the `migration` crate. The database driver feature must match the database being used.

```diff
# migration/Cargo.toml

...

[dependencies.sea-orm-migration]
version = "^0.8.0"
features = [
+   "sqlx-mysql",
+   "runtime-async-std-native-tls",
]
```

Apply all the migrations through `sea-orm-cli`:

```sh
# Change the value of DATABASE_URL according to your database implementation,
# or supply it in '.env'.
# Make sure the database name is also supplied.
$ DATABASE_URL="mysql://root:root@localhost:3306/bakeries_db" sea-orm-cli migrate refresh
```