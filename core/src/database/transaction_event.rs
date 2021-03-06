use chrono::NaiveDateTime;
use serde::{ Serialize, Deserialize };

use super::TransactionEventData;
use super::NewTransactionEventData;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TransactionEvent {
    pub id: u32,
    pub account: u32,
    pub contact: u32,
    pub sent: bool,
    pub time: NaiveDateTime,
    pub data: TransactionEventData
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct NewTransactionEvent {
    pub account: u32,
    pub contact: u32,
    pub sent: bool,
    pub time: NaiveDateTime,
    pub data: NewTransactionEventData
}

impl NewTransactionEvent {

    pub fn save(self, id: u32) -> TransactionEvent {
        match self.data {
            NewTransactionEventData::NewTextMessage { message } => {
                let data = TransactionEventData::TextMessage { message };
                TransactionEvent { id,
                    account: self.account,
                    contact: self.contact,
                    sent: self.sent,
                    time: self.time,
                    data
                }
            },
            NewTransactionEventData::NewInvitation { } => {
                let data = TransactionEventData::Invitation {};
                TransactionEvent {
                    id,
                    account: self.account,
                    contact: self.contact,
                    sent: self.sent,
                    time: self.time,
                    data
                }
            }
        }
    }
}
