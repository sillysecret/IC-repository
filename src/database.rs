use core::ascii;

use crate::{Loginpayload, Pessoa, PessoaDTS, Transaction, TransactionDTS};
use sqlx::{postgres::{PgPoolOptions, PgRow}, types::time, PgPool, Row};
use uuid::Uuid;
use time::OffsetDateTime;


pub struct Repository{
    pool: PgPool, 
}


impl Repository {
    pub async fn conn(url : String) -> Self {
        Repository{
        pool : PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .unwrap(),
        }
    }

    pub async fn createPessoa(&self  , newperson:PessoaDTS) -> Result<Pessoa, sqlx::Error>{
    let idtemp = Uuid::now_v7();
    sqlx::query_as(
        " 
        INSERT INTO Pessoa (id, name, email, CPF, balance, tipo, password)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, name, email, CPF, balance, tipo, password
        ",
    )
    .bind(idtemp)
    .bind(newperson.name)
    .bind(newperson.email)
    .bind(newperson.cpf) 
    .bind(newperson.balance)
    .bind(newperson.tipo)
    .bind(newperson.password)
    .fetch_one(&self.pool)
    .await
    
    }

   pub async fn createTransaction(&self  , newTransaction:TransactionDTS) -> Result<Transaction, sqlx::Error>{    
    
    let newid = uuid::Uuid::now_v7();
    let current_time = OffsetDateTime::now_utc();
    let formatted_time = current_time.date();
    sqlx::query_as!(
        Transaction,
        "
        INSERT INTO Transacao (id, payee, payer, amount, tempo)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, payee, payer, amount, tempo
        ",
        newid, 
        newTransaction.payee,
        newTransaction.payer,
        newTransaction.amount, 
        formatted_time // Convert OffsetDateTime to Date
    )
    .fetch_one(&self.pool)
    .await
   
    }

    pub async fn update_balance_of_payee(&self ,data:TransactionDTS) -> Result<(), sqlx::Error>{   
        print!("passou pelo update do recebedor");
        print!("{}",data.payee);
        sqlx::query_as(
            "
            UPDATE Pessoa
            SET balance = balance + $1
            WHERE id = $2
            "
        )
        .bind(data.amount)
        .bind(data.payee)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_balance_of_payer(&self ,data:TransactionDTS) -> Result<(), sqlx::Error>{   
        print!("passou pelo update do pagante");
        print!("{}",data.payer);
        print!("{}",data.payee);
        sqlx::query_as(
            "
            UPDATE Pessoa
            SET balance = balance - $1
            WHERE id = $2
            "
        )
        .bind(data.amount)
        .bind(data.payer)
        .fetch_one(&self.pool)
        .await
    }



    pub async fn searchPessoa(&self  , query: String) ->Result<Option<Pessoa>, sqlx::Error>{
        sqlx::query_as("
            SELECT * 
            FROM Pessoa 
            WHERE to_tsquery('people',$1) @@ search
            LIMIT 50
        ",
        )
        .bind(query)
        .fetch_optional(&self.pool)
        .await

    }

    pub async fn findPessoa(&self  , id:Uuid) ->Result<Option<Pessoa>, sqlx::Error>{
        sqlx::query_as("
            SELECT * FROM Pessoa WHERE id=$1
        ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await 

    } 

    pub async fn LogPessoa(&self, payload: Loginpayload) ->Result<Option<PgRow>, sqlx::Error>{
        
        //CONSULTA PARAMETRIZADA AO TESTAR MUDAR DE OPTION<PgRow> para option<Pessoa>
        // sqlx::query_as("
        //     SELECT * FROM Pessoa WHERE email=$1 AND password=$2
        // ",
        // )
        // .bind(query.email)
        // .bind(query.password)
        // .fetch_optional(&self.pool)
        // .await
        
        



        //CONSULTA N PARAMETRIZADA
        let query = format!(
            "SELECT * FROM Pessoa WHERE email = '{}' AND password = '{}'",
            payload.email, payload.password
        );
        println!("{}",payload.email);
        println!("{}",payload.password);
        println!("{}", query);

        sqlx::query(&query)
            .fetch_optional(&self.pool)
            .await

            // curl -X POST localhost:3000/loginPessoa \
            // -H "Content-Type: application/json" \
            // -d '{
            //       "email": "joao@example.com",
            //       "password": "\" 1=1 --\""
            //     }'

      
          
    }

}
