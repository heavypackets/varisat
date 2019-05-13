use std::io::{self, BufRead, Write};

use failure::Error;

use crate::lit::Lit;
use crate::vli_enc::{read_u64, write_u64};

use super::{ClauseHash, ProofStep};

const CODE_AT_CLAUSE: u64 = 0;
const CODE_UNIT_CLAUSES: u64 = 1;
const CODE_DELETE_CLAUSE: u64 = 2;

/// Writes a proof step in the varisat format
pub fn write_step<'s>(target: &mut impl Write, step: &'s ProofStep<'s>) -> io::Result<()> {
    match step {
        ProofStep::AtClause {
            clause,
            propagation_hashes,
        } => {
            write_u64(&mut *target, CODE_AT_CLAUSE)?;
            write_literals(&mut *target, clause)?;
            write_hashes(&mut *target, propagation_hashes)?;
        }

        ProofStep::UnitClauses(units) => {
            write_u64(&mut *target, CODE_UNIT_CLAUSES)?;
            write_unit_clauses(&mut *target, units)?;
        }

        ProofStep::DeleteClause(clause) => {
            write_u64(&mut *target, CODE_DELETE_CLAUSE)?;
            write_literals(&mut *target, clause)?;
        }
    }

    Ok(())
}

#[derive(Default)]
pub struct Parser {
    lit_buf: Vec<Lit>,
    hash_buf: Vec<ClauseHash>,
    unit_buf: Vec<(Lit, ClauseHash)>,
}

impl Parser {
    pub fn parse_step<'a>(&'a mut self, source: &mut impl BufRead) -> Result<ProofStep<'a>, Error> {
        match read_u64(&mut *source)? {
            CODE_AT_CLAUSE => {
                read_literals(&mut *source, &mut self.lit_buf)?;
                read_hashes(&mut *source, &mut self.hash_buf)?;
                Ok(ProofStep::AtClause {
                    clause: &self.lit_buf,
                    propagation_hashes: &self.hash_buf,
                })
            }
            CODE_UNIT_CLAUSES => {
                read_unit_clauses(&mut *source, &mut self.unit_buf)?;
                Ok(ProofStep::UnitClauses(&self.unit_buf))
            }
            CODE_DELETE_CLAUSE => {
                read_literals(&mut *source, &mut self.lit_buf)?;
                Ok(ProofStep::DeleteClause(&self.lit_buf))
            }
            _ => failure::bail!("parse error"),
        }
    }
}

/// Writes a slice of literals for a varisat proof
fn write_literals(target: &mut impl Write, literals: &[Lit]) -> io::Result<()> {
    write_u64(&mut *target, literals.len() as u64)?;
    for &lit in literals {
        write_u64(&mut *target, lit.code() as u64)?;
    }
    Ok(())
}

/// Read a slice of literals from a varisat proof
fn read_literals(source: &mut impl BufRead, literals: &mut Vec<Lit>) -> Result<(), io::Error> {
    literals.clear();
    let len = read_u64(&mut *source)? as usize;
    literals.reserve(len);
    for _ in 0..len {
        literals.push(Lit::from_code(read_u64(&mut *source)? as usize));
    }
    Ok(())
}

/// Writes a slice of clause hashes for a varisat proof
fn write_hashes(target: &mut impl Write, hashes: &[ClauseHash]) -> io::Result<()> {
    write_u64(&mut *target, hashes.len() as u64)?;
    for &hash in hashes {
        write_u64(&mut *target, hash as u64)?;
    }
    Ok(())
}

/// Read a slice of clause hashes from a varisat proof
fn read_hashes(source: &mut impl BufRead, hashes: &mut Vec<ClauseHash>) -> Result<(), io::Error> {
    hashes.clear();
    let len = read_u64(&mut *source)? as usize;
    hashes.reserve(len);
    for _ in 0..len {
        hashes.push(read_u64(&mut *source)? as ClauseHash);
    }
    Ok(())
}

/// Writes a slice of unit clauses for a varisat proof
fn write_unit_clauses(target: &mut impl Write, units: &[(Lit, ClauseHash)]) -> io::Result<()> {
    write_u64(&mut *target, units.len() as u64)?;
    for &(lit, hash) in units {
        write_u64(&mut *target, lit.code() as u64)?;
        write_u64(&mut *target, hash as u64)?;
    }
    Ok(())
}

/// Read a slice of unit clauses from a varisat proof
fn read_unit_clauses(
    source: &mut impl BufRead,
    units: &mut Vec<(Lit, ClauseHash)>,
) -> Result<(), io::Error> {
    units.clear();
    let len = read_u64(&mut *source)? as usize;
    units.reserve(len);
    for _ in 0..len {
        let lit = Lit::from_code(read_u64(&mut *source)? as usize);
        let hash = read_u64(&mut *source)? as ClauseHash;
        units.push((lit, hash));
    }
    Ok(())
}
