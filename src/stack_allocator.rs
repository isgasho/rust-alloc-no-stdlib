extern crate core;
use super::allocated_memory;
use super::allocated_stack_memory::AllocatedStackMemory;
use super::SliceWrapper;

pub trait Allocator<T> {
    type AllocatedMemory : allocated_memory::AllocatedSlice<T>;
    fn alloc_cell(&mut self, len : usize) -> Self::AllocatedMemory;
    fn free_cell(&mut self, data : Self::AllocatedMemory);
}



pub struct StackAllocator<'a,
                           T :'a,
                           U : allocated_memory::AllocatedSlice<&'a mut [T]> > {
    pub nop : &'a mut [T],
    pub system_resources : U,
    pub free_list_start : usize,
    pub free_list_overflow_count : usize,
}


impl<'a, T : 'a, U : allocated_memory::AllocatedSlice<&'a mut[T]> >
    Allocator<T> for StackAllocator <'a, T, U> {
    type AllocatedMemory = AllocatedStackMemory<'a, T>;
    fn alloc_cell(self : &mut StackAllocator<'a, T, U>,
                  len : usize) -> AllocatedStackMemory<'a, T> {
        if len == 0 {
            return AllocatedStackMemory::<'a, T>::default();
        }
        let mut index : usize = self.free_list_start;
        let mut found : bool = false;
        for free_resource in self.system_resources.slice()[self.free_list_start..].iter() {
            if free_resource.len() >= len {
                found = true;
                break;
            }
            index += 1;
        }
        if !found {
            panic!("OOM");
        }
        let mut available_slice = core::mem::replace(&mut self.system_resources.slice_mut()[index],
                                                    &mut[]);
        if available_slice.len() == len
           || (available_slice.len() < len + 32
               && index + 1 != self.system_resources.slice().len()) {
            // we don't want really small wasted slices
            // we must assign free_list_start
            if index != self.free_list_start {
                assert!(index > self.free_list_start);
                let mut farthest_free_list = core::mem::replace(
                    &mut self.system_resources.slice_mut()[self.free_list_start],
                    &mut []);
                core::mem::replace(&mut self.system_resources.slice_mut()[index],
                                   farthest_free_list);
            }
            self.free_list_start += 1;
            return AllocatedStackMemory::<'a, T>{mem:available_slice};
        } else { // the memory allocated was not the entire range of items. Split and move on
            let (mut retval, return_to_sender) = available_slice.split_at_mut(len);
            core::mem::replace(&mut self.system_resources.slice_mut()[index], return_to_sender);
            return AllocatedStackMemory::<'a, T>{mem:retval};
        }
    }
    fn free_cell(self : &mut StackAllocator<'a, T, U>,
                 mut val : AllocatedStackMemory<'a, T>) {
        if val.slice().len() == 0 {
            return;
        }
        if self.free_list_start > 0 {
            self.free_list_start -=1;
            core::mem::replace(&mut self.system_resources.slice_mut()[self.free_list_start],
                               val.mem);

        } else {
            for _i in 0..3 {
               self.free_list_overflow_count += 1;
               self.free_list_overflow_count %= self.system_resources.slice().len();
               if self.system_resources.slice()[self.free_list_overflow_count].len() < val.mem.len() {
                   core::mem::replace(&mut self.system_resources.slice_mut()[self.free_list_overflow_count],
                                      val.mem);
                   return;
               }
            }
        }
    }
}

