use super::error::{Error, ErrorKind};
use ethabi::{ParamType, Token};
use ovm::types::core::{Integer, Property};
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};
use plasma_core::data_structure::{Range, Transaction};

#[derive(Clone, Debug)]
pub struct StateUpdate {
    block_number: Integer,
    range: Range,
    property: Property,
}

impl StateUpdate {
    pub fn new(range: Range, property: Property, block_number: Integer) -> Self {
        Self {
            block_number,
            range,
            property,
        }
    }

    pub fn get_range(&self) -> Range {
        self.range
    }

    pub fn set_range(&mut self, range: Range) {
        self.range = range
    }

    pub fn get_property(&self) -> &Property {
        &self.property
    }

    pub fn get_block_number(&self) -> Integer {
        self.block_number
    }

    /// validate transaction and state update.
    pub fn execute_state_transition(&self, transaction: &Transaction) -> Result<Self, Error> {
        // TODO: switch using self.property.
        // now just transition ownership.

        Ok(Self {
            block_number: Integer::new(self.block_number.0 + 1),
            range: transaction.get_range().clone(),
            property: self.property.clone(),
        })
    }
}

impl Encodable for StateUpdate {
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Uint(self.block_number.0.into()),
            Token::Tuple(self.range.to_tuple()),
            Token::Tuple(self.property.to_tuple()),
        ]
    }
}

impl Decodable for StateUpdate {
    type Ok = StateUpdate;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let block_number = tuple[0].clone().to_uint();
        let range = tuple[1].clone().to_tuple();
        let property = tuple[2].clone().to_tuple();

        if let (Some(block_number), Some(range), Some(property)) = (block_number, range, property) {
            Ok(StateUpdate {
                block_number: Integer(block_number.as_u64()),
                range: Range::from_tuple(&range).unwrap(),
                property: Property::from_tuple(&property).unwrap(),
            })
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }

    fn get_param_types() -> Vec<ParamType> {
        vec![
            ParamType::Uint(64),
            ParamType::Tuple(Range::get_param_types()),
            ParamType::Tuple(Property::get_param_types()),
        ]
    }
}
