use std::collections::HashMap;
use std::fs;

use crate::context::{Arch, Endian};
use crate::error::{Error, Result};

const ELF_MAGIC: &[u8; 4] = b"\x7fELF";
const ELFCLASS32: u8 = 1;
const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;
const ELFDATA2MSB: u8 = 2;

const EM_386: u16 = 3;
const EM_MIPS: u16 = 8;
const EM_PPC: u16 = 20;
const EM_PPC64: u16 = 21;
const EM_ARM: u16 = 40;
const EM_X86_64: u16 = 62;
const EM_AARCH64: u16 = 183;
const EM_RISCV: u16 = 243;

const PT_LOAD: u32 = 1;
const PF_X: u32 = 1;

const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_RELA: u32 = 4;
const SHT_DYNSYM: u32 = 11;
const SHT_REL: u32 = 9;

pub struct ELF {
    pub data: Vec<u8>,
    pub path: String,
    symbols: HashMap<String, u64>,
    got: HashMap<String, u64>,
    plt: HashMap<String, u64>,
    pub base_offset: i64,
    executable_segments: Vec<(u64, Vec<u8>)>,
    arch: Arch,
    bits: u32,
    endian: Endian,
    entry: u64,
    base_address: u64,
}

#[derive(Clone, Copy)]
struct Header {
    bits: u32,
    endian: Endian,
    machine: u16,
    entry: u64,
    phoff: u64,
    shoff: u64,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[derive(Clone, Copy)]
struct SectionHeader {
    name: u32,
    typ: u32,
    addr: u64,
    offset: u64,
    size: u64,
    link: u32,
    entsize: u64,
}

#[derive(Clone, Copy)]
struct ProgramHeader {
    typ: u32,
    flags: u32,
    offset: u64,
    vaddr: u64,
    filesz: u64,
}

impl ELF {
    pub fn new(path: &str) -> Result<Self> {
        let data = fs::read(path)?;
        let (arch, bits, endian, entry, symbols, got, plt, executable_segments, base_address) = {
            let header = parse_header(&data)?;
            let arch = parse_arch(header.machine, header.bits)?;
            let section_headers = parse_section_headers(&data, header)?;
            let program_headers = parse_program_headers(&data, header)?;
            let section_names = parse_section_names(&data, &section_headers, header)?;
            let symbols = parse_symbols(&data, &section_headers, header)?;
            let (got, plt) = parse_plt_data(&data, &section_headers, &section_names, header)?;
            let (executable_segments, base_address) = parse_segments(&data, &program_headers)?;
            (
                arch,
                header.bits,
                header.endian,
                header.entry,
                symbols,
                got,
                plt,
                executable_segments,
                base_address,
            )
        };

        Ok(Self {
            data,
            path: path.to_string(),
            symbols,
            got,
            plt,
            base_offset: 0,
            executable_segments,
            arch,
            bits,
            endian,
            entry,
            base_address,
        })
    }

    pub fn sym(&self, name: &str) -> u64 {
        self.try_sym(name)
            .unwrap_or_else(|| panic!("symbol not found: {name}"))
    }

    pub fn got(&self, name: &str) -> u64 {
        self.try_got(name)
            .unwrap_or_else(|| panic!("GOT entry not found: {name}"))
    }

    pub fn plt(&self, name: &str) -> u64 {
        self.try_plt(name)
            .unwrap_or_else(|| panic!("PLT entry not found: {name}"))
    }

    pub fn try_sym(&self, name: &str) -> Option<u64> {
        self.symbols
            .get(name)
            .copied()
            .map(|addr| self.rebase(addr))
    }

    pub fn try_got(&self, name: &str) -> Option<u64> {
        self.got.get(name).copied().map(|addr| self.rebase(addr))
    }

    pub fn try_plt(&self, name: &str) -> Option<u64> {
        self.plt.get(name).copied().map(|addr| self.rebase(addr))
    }

    pub fn entry(&self) -> u64 {
        self.rebase(self.entry)
    }

    pub fn address(&self) -> u64 {
        self.rebase(self.base_address)
    }

    pub fn set_base(&mut self, addr: u64) {
        self.base_offset = addr as i64 - self.base_address as i64;
    }

    pub fn search(&self, needle: &[u8]) -> Option<u64> {
        if needle.is_empty() {
            return None;
        }
        for (vaddr, segment) in &self.executable_segments {
            if needle.len() > segment.len() {
                continue;
            }
            for index in 0..=segment.len() - needle.len() {
                if &segment[index..index + needle.len()] == needle {
                    return Some(self.rebase(*vaddr + index as u64));
                }
            }
        }
        None
    }

    pub fn executable_segments(&self) -> &[(u64, Vec<u8>)] {
        &self.executable_segments
    }

    pub fn arch(&self) -> Arch {
        self.arch
    }

    pub fn bits(&self) -> u32 {
        self.bits
    }

    pub(crate) fn endian(&self) -> Endian {
        self.endian
    }

    fn rebase(&self, addr: u64) -> u64 {
        if self.base_offset >= 0 {
            addr + self.base_offset as u64
        } else {
            addr - self.base_offset.unsigned_abs()
        }
    }
}

fn parse_header(data: &[u8]) -> Result<Header> {
    if data.len() < 16 || data.get(0..4) != Some(ELF_MAGIC.as_slice()) {
        return Err(Error::other("invalid ELF magic"));
    }

    let bits = match data[4] {
        ELFCLASS32 => 32,
        ELFCLASS64 => 64,
        other => return Err(Error::other(format!("unsupported ELF class {other}"))),
    };
    let endian = match data[5] {
        ELFDATA2LSB => Endian::Little,
        ELFDATA2MSB => Endian::Big,
        other => {
            return Err(Error::other(format!(
                "unsupported ELF data encoding {other}"
            )))
        }
    };

    let machine = r16(data, 0x12, endian)?;
    let (entry, phoff, shoff, phentsize, phnum, shentsize, shnum, shstrndx) = if bits == 64 {
        (
            r64(data, 0x18, endian)?,
            r64(data, 0x20, endian)?,
            r64(data, 0x28, endian)?,
            r16(data, 0x36, endian)?,
            r16(data, 0x38, endian)?,
            r16(data, 0x3a, endian)?,
            r16(data, 0x3c, endian)?,
            r16(data, 0x3e, endian)?,
        )
    } else {
        (
            r32(data, 0x18, endian)? as u64,
            r32(data, 0x1c, endian)? as u64,
            r32(data, 0x20, endian)? as u64,
            r16(data, 0x2a, endian)?,
            r16(data, 0x2c, endian)?,
            r16(data, 0x2e, endian)?,
            r16(data, 0x30, endian)?,
            r16(data, 0x32, endian)?,
        )
    };

    Ok(Header {
        bits,
        endian,
        machine,
        entry,
        phoff,
        shoff,
        phentsize,
        phnum,
        shentsize,
        shnum,
        shstrndx,
    })
}

fn parse_arch(machine: u16, bits: u32) -> Result<Arch> {
    match (machine, bits) {
        (EM_X86_64, 64) => Ok(Arch::Amd64),
        (EM_386, 32) => Ok(Arch::I386),
        (EM_ARM, 32) => Ok(Arch::Arm),
        (EM_AARCH64, 64) => Ok(Arch::Aarch64),
        (EM_MIPS, 64) => Ok(Arch::Mips64),
        (EM_MIPS, _) => Ok(Arch::Mips),
        (EM_PPC, 32) => Ok(Arch::Ppc),
        (EM_PPC64, 64) => Ok(Arch::Ppc64),
        (EM_RISCV, 64) => Ok(Arch::Riscv64),
        (EM_RISCV, _) => Ok(Arch::Riscv32),
        _ => Err(Error::other(format!("unsupported ELF machine {machine}"))),
    }
}

fn parse_section_headers(data: &[u8], header: Header) -> Result<Vec<SectionHeader>> {
    if header.shoff == 0 || header.shnum == 0 {
        return Ok(Vec::new());
    }
    let expected_size = if header.bits == 64 { 64 } else { 40 };
    if header.shentsize < expected_size {
        return Err(Error::other("invalid section header size"));
    }

    let mut sections = Vec::with_capacity(header.shnum as usize);
    for index in 0..header.shnum as usize {
        let offset = table_offset(header.shoff, header.shentsize, index)?;
        let section = if header.bits == 64 {
            SectionHeader {
                name: r32(data, offset, header.endian)?,
                typ: r32(data, offset + 4, header.endian)?,
                addr: r64(data, offset + 16, header.endian)?,
                offset: r64(data, offset + 24, header.endian)?,
                size: r64(data, offset + 32, header.endian)?,
                link: r32(data, offset + 40, header.endian)?,
                entsize: r64(data, offset + 56, header.endian)?,
            }
        } else {
            SectionHeader {
                name: r32(data, offset, header.endian)?,
                typ: r32(data, offset + 4, header.endian)?,
                addr: r32(data, offset + 12, header.endian)? as u64,
                offset: r32(data, offset + 16, header.endian)? as u64,
                size: r32(data, offset + 20, header.endian)? as u64,
                link: r32(data, offset + 24, header.endian)?,
                entsize: r32(data, offset + 36, header.endian)? as u64,
            }
        };
        sections.push(section);
    }
    Ok(sections)
}

fn parse_program_headers(data: &[u8], header: Header) -> Result<Vec<ProgramHeader>> {
    if header.phoff == 0 || header.phnum == 0 {
        return Ok(Vec::new());
    }
    let expected_size = if header.bits == 64 { 56 } else { 32 };
    if header.phentsize < expected_size {
        return Err(Error::other("invalid program header size"));
    }

    let mut programs = Vec::with_capacity(header.phnum as usize);
    for index in 0..header.phnum as usize {
        let offset = table_offset(header.phoff, header.phentsize, index)?;
        let program = if header.bits == 64 {
            ProgramHeader {
                typ: r32(data, offset, header.endian)?,
                flags: r32(data, offset + 4, header.endian)?,
                offset: r64(data, offset + 8, header.endian)?,
                vaddr: r64(data, offset + 16, header.endian)?,
                filesz: r64(data, offset + 32, header.endian)?,
            }
        } else {
            ProgramHeader {
                typ: r32(data, offset, header.endian)?,
                offset: r32(data, offset + 4, header.endian)? as u64,
                vaddr: r32(data, offset + 8, header.endian)? as u64,
                filesz: r32(data, offset + 16, header.endian)? as u64,
                flags: r32(data, offset + 24, header.endian)?,
            }
        };
        programs.push(program);
    }
    Ok(programs)
}

fn parse_section_names(
    data: &[u8],
    sections: &[SectionHeader],
    header: Header,
) -> Result<Vec<String>> {
    if sections.is_empty() {
        return Ok(Vec::new());
    }
    let Some(strtab_section) = sections.get(header.shstrndx as usize) else {
        return Ok(vec![String::new(); sections.len()]);
    };
    let strtab = section_data(data, *strtab_section)?;

    let mut names = Vec::with_capacity(sections.len());
    for section in sections {
        names.push(read_string(strtab, section.name as usize).unwrap_or_default());
    }
    Ok(names)
}

fn parse_symbols(
    data: &[u8],
    sections: &[SectionHeader],
    header: Header,
) -> Result<HashMap<String, u64>> {
    let mut symbols = HashMap::new();

    for &section_type in &[SHT_SYMTAB, SHT_DYNSYM] {
        for section in sections
            .iter()
            .copied()
            .filter(|section| section.typ == section_type)
        {
            let Some(strtab_section) = sections.get(section.link as usize).copied() else {
                continue;
            };
            if strtab_section.typ != SHT_STRTAB {
                continue;
            }

            let symtab = section_data(data, section)?;
            let strtab = section_data(data, strtab_section)?;
            let default_entsize = if header.bits == 64 { 24 } else { 16 };
            let entsize = if section.entsize == 0 {
                default_entsize
            } else {
                section.entsize
            } as usize;
            if entsize < default_entsize as usize || entsize == 0 {
                continue;
            }

            for offset in (0..symtab.len()).step_by(entsize) {
                let Some(entry) = symtab.get(offset..offset + entsize) else {
                    break;
                };
                let (name_offset, value) = if header.bits == 64 {
                    (
                        r32(entry, 0, header.endian)? as usize,
                        r64(entry, 8, header.endian)?,
                    )
                } else {
                    (
                        r32(entry, 0, header.endian)? as usize,
                        r32(entry, 4, header.endian)? as u64,
                    )
                };
                if value == 0 {
                    continue;
                }
                let Some(name) = read_string(strtab, name_offset) else {
                    continue;
                };
                if !name.is_empty() {
                    symbols.insert(name, value);
                }
            }
        }
    }

    Ok(symbols)
}

fn parse_plt_data(
    data: &[u8],
    sections: &[SectionHeader],
    section_names: &[String],
    header: Header,
) -> Result<(HashMap<String, u64>, HashMap<String, u64>)> {
    let mut reloc_names = Vec::new();

    for (index, section) in sections.iter().copied().enumerate() {
        let Some(name) = section_names.get(index) else {
            continue;
        };
        if name != ".rela.plt" && name != ".rel.plt" {
            continue;
        }

        let Some(symtab_section) = sections.get(section.link as usize).copied() else {
            continue;
        };
        let Some(strtab_section) = sections.get(symtab_section.link as usize).copied() else {
            continue;
        };
        let symtab = section_data(data, symtab_section)?;
        let strtab = section_data(data, strtab_section)?;
        let relocs = section_data(data, section)?;

        let default_entsize = match (section.typ, header.bits) {
            (SHT_RELA, 64) => 24,
            (SHT_RELA, 32) => 12,
            (SHT_REL, 64) => 16,
            (SHT_REL, 32) => 8,
            _ => continue,
        };
        let reloc_entsize = if section.entsize == 0 {
            default_entsize
        } else {
            section.entsize
        } as usize;
        if reloc_entsize < default_entsize as usize || reloc_entsize == 0 {
            continue;
        }

        for offset in (0..relocs.len()).step_by(reloc_entsize) {
            let Some(entry) = relocs.get(offset..offset + reloc_entsize) else {
                break;
            };
            let (r_offset, sym_index) = if header.bits == 64 {
                let r_offset = r64(entry, 0, header.endian)?;
                let r_info = r64(entry, 8, header.endian)?;
                (r_offset, (r_info >> 32) as usize)
            } else {
                let r_offset = r32(entry, 0, header.endian)? as u64;
                let r_info = r32(entry, 4, header.endian)?;
                (r_offset, (r_info >> 8) as usize)
            };
            let Some(symbol_name) = symbol_name(symtab, strtab, sym_index, header)? else {
                continue;
            };
            if !symbol_name.is_empty() {
                reloc_names.push((symbol_name, r_offset));
            }
        }
    }

    let mut got = HashMap::new();
    for (name, offset) in &reloc_names {
        got.insert(name.clone(), *offset);
    }

    let mut plt = HashMap::new();
    let plt_section = sections
        .iter()
        .copied()
        .zip(section_names.iter())
        .find(|(_, name)| name.as_str() == ".plt.sec")
        .or_else(|| {
            sections
                .iter()
                .copied()
                .zip(section_names.iter())
                .find(|(_, name)| name.as_str() == ".plt")
        });
    if let Some((section, name)) = plt_section {
        let base = section.addr;
        let entry_size = if section.entsize == 0 {
            16
        } else {
            section.entsize
        };
        let is_plt_sec = name == ".plt.sec";
        for (index, (symbol_name, _)) in reloc_names.iter().enumerate() {
            let slot = if is_plt_sec {
                base + index as u64 * entry_size
            } else {
                base + (index as u64 + 1) * entry_size
            };
            plt.insert(symbol_name.clone(), slot);
        }
    }

    Ok((got, plt))
}

fn parse_segments(data: &[u8], programs: &[ProgramHeader]) -> Result<(Vec<(u64, Vec<u8>)>, u64)> {
    let mut executable_segments = Vec::new();
    let mut base_address = 0u64;
    let mut first_load = true;

    for program in programs {
        if program.typ != PT_LOAD {
            continue;
        }
        if first_load {
            base_address = program.vaddr;
            first_load = false;
        }
        if program.flags & PF_X == 0 {
            continue;
        }

        let start = usize::try_from(program.offset)
            .map_err(|_| Error::other("segment offset does not fit in memory"))?;
        let size = usize::try_from(program.filesz)
            .map_err(|_| Error::other("segment size does not fit in memory"))?;
        let Some(end) = start.checked_add(size) else {
            continue;
        };
        let Some(bytes) = data.get(start..end) else {
            continue;
        };
        executable_segments.push((program.vaddr, bytes.to_vec()));
    }

    Ok((executable_segments, base_address))
}

fn symbol_name(
    symtab: &[u8],
    strtab: &[u8],
    index: usize,
    header: Header,
) -> Result<Option<String>> {
    let entsize = if header.bits == 64 { 24 } else { 16 };
    let Some(offset) = index.checked_mul(entsize) else {
        return Ok(None);
    };
    let Some(entry) = symtab.get(offset..offset + entsize) else {
        return Ok(None);
    };
    let name_offset = r32(entry, 0, header.endian)? as usize;
    Ok(read_string(strtab, name_offset))
}

fn section_data<'a>(data: &'a [u8], section: SectionHeader) -> Result<&'a [u8]> {
    let start = usize::try_from(section.offset)
        .map_err(|_| Error::other("section offset does not fit in memory"))?;
    let size = usize::try_from(section.size)
        .map_err(|_| Error::other("section size does not fit in memory"))?;
    let end = start
        .checked_add(size)
        .ok_or_else(|| Error::other("section range overflow"))?;
    data.get(start..end)
        .ok_or_else(|| Error::other("section extends past end of file"))
}

fn read_string(data: &[u8], offset: usize) -> Option<String> {
    let tail = data.get(offset..)?;
    let end = tail.iter().position(|&byte| byte == 0)?;
    Some(String::from_utf8_lossy(&tail[..end]).into_owned())
}

fn table_offset(base: u64, entsize: u16, index: usize) -> Result<usize> {
    let index = u64::try_from(index).map_err(|_| Error::other("table index overflow"))?;
    let offset = base
        .checked_add(index.saturating_mul(u64::from(entsize)))
        .ok_or_else(|| Error::other("table offset overflow"))?;
    usize::try_from(offset).map_err(|_| Error::other("table offset does not fit in memory"))
}

fn bytes<const N: usize>(data: &[u8], offset: usize) -> Result<[u8; N]> {
    let end = offset
        .checked_add(N)
        .ok_or_else(|| Error::other("ELF read overflow"))?;
    let slice = data
        .get(offset..end)
        .ok_or_else(|| Error::other("ELF truncated"))?;
    let mut out = [0u8; N];
    out.copy_from_slice(slice);
    Ok(out)
}

fn r16(data: &[u8], offset: usize, endian: Endian) -> Result<u16> {
    let bytes = bytes::<2>(data, offset)?;
    Ok(match endian {
        Endian::Little => u16::from_le_bytes(bytes),
        Endian::Big => u16::from_be_bytes(bytes),
    })
}

fn r32(data: &[u8], offset: usize, endian: Endian) -> Result<u32> {
    let bytes = bytes::<4>(data, offset)?;
    Ok(match endian {
        Endian::Little => u32::from_le_bytes(bytes),
        Endian::Big => u32::from_be_bytes(bytes),
    })
}

fn r64(data: &[u8], offset: usize, endian: Endian) -> Result<u64> {
    let bytes = bytes::<8>(data, offset)?;
    Ok(match endian {
        Endian::Little => u64::from_le_bytes(bytes),
        Endian::Big => u64::from_be_bytes(bytes),
    })
}
