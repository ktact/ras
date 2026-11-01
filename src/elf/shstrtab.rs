use super::strtab::StrTab;

pub struct ShStrTab {
    strtab: StrTab,
}

impl ShStrTab {
    pub fn new(section_names: &[&str]) -> Self {
        let mut strtab = StrTab::new();
        for name in section_names {
            strtab.add_str(name);
        }
        Self { strtab }
    }

    pub fn get_offset_by(&self, name: &str) -> Option<u32> {
        self.strtab.get_offset_by(name)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.strtab.as_bytes()
    }
}
