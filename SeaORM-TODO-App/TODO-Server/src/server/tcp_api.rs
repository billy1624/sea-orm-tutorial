use crate::{Fruits, Suppliers, Todos, TodosActiveModel, TodosColumn, TodosModel};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

// The commands to use to perform CRUD operations on PostgreSQL
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Store { username: String, todo_list: String },
    UpdateTodoList { username: String, todo_list: String },
    Get(String),
    ListFruits,
    ListSuppliers,
    DeleteUser(String),
}

impl Command {
    pub async fn get_fruits(db: &DatabaseConnection) -> anyhow::Result<Vec<u8>> {
        let fruit_models = Fruits::find().all(db).await?;
        let fruits = fruit_models
            .iter()
            .map(|fruit_model| fruit_model.fruit_name.clone())
            .collect::<String>();

        Ok(bincode::serialize(&fruits)?)
    }
    pub async fn get_suppliers(db: &DatabaseConnection) -> anyhow::Result<Vec<u8>> {
        let supplier_models = Suppliers::find().all(db).await?;
        let suppliers = supplier_models
            .iter()
            .map(|supplier_model| supplier_model.suppliers_name.clone())
            .collect::<String>();

        Ok(bincode::serialize(&suppliers)?)
    }

    pub async fn store(&self, db: &DatabaseConnection) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Store {
                username,
                todo_list,
            } => {
                let todo_user = TodosActiveModel {
                    username: Set(username.to_owned()),
                    todo_list: Set(Some(todo_list.to_owned())),
                    ..Default::default()
                };
                Todos::insert(todo_user).exec(db).await?;

                Ok(bincode::serialize("INSERTED")?)
            }
            _ => Err(anyhow::Error::new(ServerErrors::InvalidCommand)),
        }
    }

    pub async fn get_user_todo(&self, db: &DatabaseConnection) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Get(user) => {
                let found_todo = Todos::find()
                    .filter(TodosColumn::Username.contains(user))
                    .all(db)
                    .await?;

                let mut todo_list = Vec::default();

                found_todo.iter().for_each(|todo_model| {
                    if let Some(todo_exists) = &todo_model.todo_list {
                        todo_list.push(todo_exists);
                    }
                });

                Ok(bincode::serialize(&todo_list)?)
            }
            _ => Err(anyhow::Error::new(ServerErrors::InvalidCommand)),
        }
    }

    pub async fn update_todo_list(&self, db: &DatabaseConnection) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::UpdateTodoList {
                username,
                todo_list,
            } => {
                let found_todo: Option<TodosModel> = Todos::find()
                    .filter(TodosColumn::Username.contains(username))
                    .one(db)
                    .await?;

                match found_todo {
                    Some(todo_model) => {
                        let mut todo_model: TodosActiveModel = todo_model.into();
                        todo_model.todo_list = Set(Some(todo_list.to_owned()));
                        todo_model.update(db).await?;
                    }
                    None => return Err(anyhow::Error::new(ServerErrors::ModelNotFound)),
                };

                Ok(bincode::serialize("UPDATED_TODO")?)
            }
            _ => Err(anyhow::Error::new(ServerErrors::InvalidCommand)),
        }
    }

    pub async fn delete_user(&self, db: &DatabaseConnection) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::DeleteUser(user) => {
                let found_todo: Option<TodosModel> = Todos::find()
                    .filter(TodosColumn::Username.contains(user))
                    .one(db)
                    .await?;

                match found_todo {
                    Some(todo_model) => {
                        todo_model.delete(db).await?;
                    }
                    None => return Err(anyhow::Error::new(ServerErrors::ModelNotFound)),
                };

                Ok(bincode::serialize("DELETED_USER")?)
            }
            _ => Err(anyhow::Error::new(ServerErrors::InvalidCommand)),
        }
    }
}

#[derive(Debug)]
pub enum ServerErrors {
    InvalidCommand,
    ModelNotFound,
}

impl Error for ServerErrors {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&crate::ServerErrors::InvalidCommand)
    }
}

impl fmt::Display for ServerErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}",
            match self {
                ServerErrors::InvalidCommand => "Invalid command provided",
                ModelNotFound => "The result of the query is `None`",
            }
        )
    }
}