use axum::{
    routing::{get, post},
    Router,
    response::IntoResponse,extract::{State,Path}, Json, http::StatusCode,
};

use json::JsonValue::Null;
use reqwest::Method;
use serde::{Serialize,Deserialize};
use uuid::Uuid;
use std::env;
use std::sync::Arc;
use database::Repository;
use sqlx::{postgres::PgRow, Row};
use tower_http::cors::{CorsLayer,Any};

impl From<PgRow> for Pessoa {
    fn from(row: PgRow) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            cpf: row.get("cpf"),
            balance: row.get("balance"),
            tipo: row.get("tipo"),
            password: row.get("password"),
        }
    }
}

mod database;

type AppState = Arc<Repository>;

time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Deserialize)]
pub struct Querysearch{
    pub query: String
}


#[derive(Serialize,Clone,Deserialize,sqlx::FromRow)]
pub struct Pessoa{
    pub id :Uuid,
    pub name:String,
    pub email:String,
    pub cpf:String,
    pub balance:i32,
    pub tipo:bool,
    pub password:String
}


#[derive(Serialize,Clone,Deserialize,sqlx::FromRow)]
pub struct PessoaDTS{
    pub name: String,
    pub email:String,
    pub cpf:String,
    pub balance:i32,
    pub tipo:bool,
    pub password:String
}
#[derive(Serialize,Clone,Deserialize,sqlx::FromRow)]
pub struct Loginpayload{
    pub email:String,
    pub password:String
}


#[derive(Serialize,Clone,Deserialize,sqlx::FromRow)]
pub struct Transaction{
    pub id:Uuid,
    pub payee:Uuid,
    pub payer:Uuid,
    pub amount:i32,
    #[serde(with = "date_format" )]
    pub tempo:time::Date
}

#[derive(Serialize,Clone,Deserialize,sqlx::FromRow)]
pub struct TransactionDTS{
    pub payee:Uuid,
    pub payer:Uuid,
    pub amount:i32
}

// true = cliente , false = logista  
// Nome Completo, CPF, e-mail e Senha. CPF/CNPJ e e-mails devem ser únicos no sistema.
// Sendo assim, seu sistema deve permitir apenas um cadastro com o mesmo CPF ou endereço de e-mail.


#[tokio::main]
async fn main() {
    
    // build our application with a single route
    
    let port =env::var("DATABASE_URL")
        .unwrap_or(String::from("postgres://pic:pay@localhost:5432/picpay"));
   
    let db = Repository::conn(port).await;

    let app_state = Arc::new(db);
 

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_private_network(true)
        .allow_origin(Any);

    let app = Router::new()
        .route("/transaction",post(mktransaction))
        .route("/pessoa/:id",get(findp))
        .route("/pessoa", post(mkpessoa))
        .route("/loginPessoa", post(logpessoa))
        .layer(cors)
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn mktransaction(State(localbd): State<AppState>,Json(payload): Json<TransactionDTS>)-> impl IntoResponse{        
    let payee = localbd.findPessoa(payload.payee).await;
    let payer = localbd.findPessoa(payload.payer).await; 
    
    if let (Ok(Some(pessoa1)), Ok(Some(pessoa2))) = (payee, payer) {
        // Ambas as funções retornaram uma pessoa\
        if pessoa1.tipo {
            return Err((StatusCode::UNPROCESSABLE_ENTITY,"Logista nao pode fazer transferencias"));
        }

        if pessoa2.balance<payload.amount{
            return Err((StatusCode::UNPROCESSABLE_ENTITY,"Seu balance nao suporta a transaction"));
        }     
            match fetch_data().await {
            Err(err) => eprintln!("Erro ao fazer a solicitação: {}", err),
            _=>(),
        }  
    }else {
        return Err((StatusCode::NOT_FOUND,"Alguma das pessoas nao existe"));
    }

    match localbd.createTransaction(payload.clone()).await{
        Ok(Transaction)=>{
            localbd.update_balance_of_payee(payload.clone()).await;
            localbd.update_balance_of_payer(payload.clone()).await;
            Ok((StatusCode::CREATED,Json(Transaction)))
        },
        Err(e)=>{
            println!("{:?}",e);
            Err((StatusCode::INTERNAL_SERVER_ERROR,"Erro"))
        },
    }

    


    
}

async fn mkpessoa(State(localbd): State<AppState>,Json(payload): Json<PessoaDTS>) -> impl IntoResponse {
    if payload.name.len() > 100 || payload.cpf.len()>100 || payload.password.len()>100{
           return Err((StatusCode::UNPROCESSABLE_ENTITY,"tamanho de campo invalido"));
    }
    
    if payload.balance<0{
        return Err((StatusCode::UNPROCESSABLE_ENTITY,"balance negativo"));
    }


    match localbd.createPessoa(payload).await {
        Ok(pessoa)=>Ok((StatusCode::CREATED,Json(pessoa))),
        
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() =>Err((StatusCode::UNPROCESSABLE_ENTITY,"Unnique violation")),
        
        Err(_)=>Err((StatusCode::INTERNAL_SERVER_ERROR,"INTERNAL_SERVER_ERROR")),
    
    }

  
} 
 

async fn findp(State(localbd): State<AppState>,Path(id): Path<Uuid>,) -> impl IntoResponse {
    match localbd.findPessoa(id).await {
        Ok(Some(pessoa)) =>Ok((StatusCode::OK,Json(pessoa))),
        Ok(None)=>Err((StatusCode::NOT_FOUND,"Pessoa nao encontrada")),
        Err(_)=> Err((StatusCode::INTERNAL_SERVER_ERROR,"Erro")),
    } 
}


async fn logpessoa(State(localbd): State<AppState>,Json(payload): Json<Loginpayload>) -> impl IntoResponse {
    //caso sucetivel a injecao de sql
    match localbd.LogPessoa(payload).await{
        Ok(Some(pessoa)) =>Ok((StatusCode::OK,Json(Pessoa::from(pessoa)))),
        Ok(None)=>Err((StatusCode::NOT_FOUND,"Pessoa nao encontrada")),
        Err(_)=> Err((StatusCode::INTERNAL_SERVER_ERROR,"Erro")),
    }

}




async fn fetch_data() -> Result<String, String> {
    let response = reqwest::get("https://run.mocky.io/v3/5794d450-d2e2-4412-8131-73d0293ac1cc").await;

    match response {
        Ok(response) => {
            if response.status().is_success() {
                let body = response.text().await.map_err(|e| e.to_string())?;
                Ok(body)
            } else {
                let status_code = response.status();
                let error_message = response.text().await.unwrap_or_else(|_| String::from("Unknown error"));
                let err = format!("Erro HTTP: {} - {}", status_code, error_message);
                Err(err)
            }
        }
        Err(err) => Err(err.to_string()),
    }
} 
