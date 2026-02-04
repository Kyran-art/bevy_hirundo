use crate::internal_prelude::*;

#[derive(Resource, FromWorld)]
pub struct MeshTagAllocator {
    pub next_tag: u32,
    pub free_list: VecDeque<u32>,
}

impl MeshTagAllocator {
    pub fn new() -> Self {
        MeshTagAllocator {
            next_tag: 0,
            free_list: VecDeque::new(),
        }
    }

    pub fn allocate_tag(&mut self) -> MeshTag {
        if let Some(recycled_tag) = self.free_list.pop_front() {
            MeshTag(recycled_tag)
        } else {
            let tag = self.next_tag;
            self.next_tag += 1;
            MeshTag(tag)
        }
    }

    pub fn free_tag(&mut self, tag: u32) {
        self.free_list.push_back(tag);
    }
}
