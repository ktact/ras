pub struct ShStrTab {
    pub data: Vec<u8>,
    pub offsets: Vec<(String, u32)>,
}

impl ShStrTab {
    pub fn new(section_names: &[&str]) -> Self {
        let mut data = vec![0];
        let mut offsets = Vec::new();

        for name in section_names {
            let offset = data.len() as u32;
            offsets.push((name.to_string().clone(), offset));


            data.extend_from_slice(name.as_bytes());
            data.push(0x00);
        }

        Self { data, offsets }
    }

    pub fn get_offset_by(&self, name: &str) -> Option<u32> {
        self.offsets.iter()
            .find(|(section_name, _)| section_name == name)
            .map(|(_, offset)| *offset)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}
