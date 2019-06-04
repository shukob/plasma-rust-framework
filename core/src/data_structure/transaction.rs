extern crate ethereum_types;
extern crate rlp;
extern crate tiny_keccak;

use super::error::{Error, ErrorKind};
use ethabi::Token;
use ethereum_types::{Address, H256};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use tiny_keccak::Keccak;

#[derive(Clone, Debug)]
pub struct Witness {
    v: H256,
    r: H256,
    s: u64,
}

impl Witness {
    pub fn new(v: H256, r: H256, s: u64) -> Self {
        Witness { v, r, s }
    }
}

#[derive(Clone, Debug)]
pub struct Transaction {
    plasma_contract_address: Address,
    start: u64,
    end: u64,
    method_id: u8,
    parameters: Vec<u8>,
    witness: Witness,
}

impl Transaction {
    pub fn new(
        plasma_contract_address: Address,
        start: u64,
        end: u64,
        method_id: u8,
        parameters: &[u8],
        witness: &Witness,
    ) -> Transaction {
        Transaction {
            plasma_contract_address,
            start,
            end,
            method_id,
            parameters: parameters.to_vec(),
            witness: witness.clone(),
        }
    }
    pub fn to_body_abi(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Address(self.plasma_contract_address),
            Token::Uint(self.start.into()),
            Token::Uint(self.end.into()),
            Token::FixedBytes(vec![self.method_id]),
            Token::Bytes(self.parameters.clone()),
        ])
    }
    pub fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Address(self.plasma_contract_address),
            Token::Uint(self.start.into()),
            Token::Uint(self.end.into()),
            Token::FixedBytes(vec![self.method_id]),
            Token::Bytes(self.parameters.clone()),
            Token::FixedBytes(self.witness.v.as_bytes().to_vec()),
            Token::FixedBytes(self.witness.r.as_bytes().to_vec()),
            Token::Uint(self.witness.s.into()),
        ])
    }
    pub fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded = ethabi::decode(
            &[
                ethabi::ParamType::Address,
                ethabi::ParamType::Uint(16),
                ethabi::ParamType::Uint(16),
                ethabi::ParamType::FixedBytes(1),
                ethabi::ParamType::Bytes,
                ethabi::ParamType::FixedBytes(32),
                ethabi::ParamType::FixedBytes(32),
                ethabi::ParamType::Uint(1),
            ],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        let plasma_contract = decoded[0].clone().to_address();
        let start = decoded[1].clone().to_uint();
        let end = decoded[2].clone().to_uint();
        let method_id_opt = decoded[3].clone().to_fixed_bytes();
        let parameters = decoded[4].clone().to_bytes();
        let v = decoded[5].clone().to_fixed_bytes();
        let r = decoded[6].clone().to_fixed_bytes();
        let s = decoded[7].clone().to_uint();
        if let (
            Some(plasma_contract),
            Some(start),
            Some(end),
            Some(method_id),
            Some(parameters),
            Some(v),
            Some(r),
            Some(s),
        ) = (
            plasma_contract,
            start,
            end,
            method_id_opt,
            parameters,
            v,
            r,
            s,
        ) {
            Ok(Transaction::new(
                plasma_contract,
                start.as_u64(),
                end.as_u64(),
                method_id[0],
                &parameters,
                &Witness::new(H256::from_slice(&v), H256::from_slice(&r), s.as_u64()),
            ))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
    pub fn create_method_id(value: &[u8]) -> u8 {
        let mut hasher = Keccak::new_sha3_256();
        hasher.update(value);
        let mut result: [u8; 32] = [0; 32];
        hasher.finalize(&mut result);
        result[0]
    }
    pub fn get_start(&self) -> u64 {
        self.start
    }
    pub fn get_end(&self) -> u64 {
        self.end
    }
}

impl Encodable for Transaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(1);
        s.append(&self.to_abi());
    }
}

impl Decodable for Transaction {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let abi: Vec<u8> = rlp.list_at(0)?;
        Ok(Transaction::from_abi(&abi).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::Transaction;
    use super::Witness;
    use ethereum_types::{Address, H256};

    #[test]
    fn test_abi_encode() {
        let parameters_bytes = Vec::from(&b"parameters"[..]);
        let transaction = Transaction::new(
            Address::zero(),
            0,
            100,
            Transaction::create_method_id(&b"send(address)"[..]),
            &parameters_bytes,
            &Witness::new(H256::zero(), H256::zero(), 0),
        );
        let encoded = transaction.to_abi();
        let decoded: Transaction = Transaction::from_abi(&encoded).unwrap();
        assert_eq!(decoded.start, transaction.start);
    }

}
