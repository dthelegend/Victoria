use nalgebra::SMatrix;
use usbd_human_interface_device::page::Keyboard;
use crate::keyboard::ActivationMatrix;

mod consts {
}

pub type Keymap<const NRows: usize, const NCols: usize> = SMatrix<Keyboard, NRows, NCols>;


pub trait LayeredKeymap<const NRows: usize, const NCols: usize>
{
    fn get_map(&self, activation_matrix : &ActivationMatrix<NRows, NCols>) -> impl IntoIterator<Item = Keyboard>;
}

pub struct DefaultKeymap<'a, const NRows: usize, const NCols: usize> {
    default_keymap: Keymap<NRows, NCols>,
    function_key: (usize, usize),
    function_keymap: Keymap<NRows, NCols>,
    hardware_break : &'a [(usize, usize)]
}

impl <'a, const NRows: usize, const NCols: usize> DefaultKeymap<'a, NRows, NCols> {
    
    pub const fn new(default_keymap: Keymap<NRows, NCols>, function_key: (usize, usize), function_keymap: Keymap<NRows, NCols>, hardware_break : &'a [(usize, usize)]) -> Self {
        assert!(function_key.0 < NRows && function_key.1 < NCols);
        Self {
            default_keymap,
            function_key,
            function_keymap,
            hardware_break
        }
    }
}

impl <'a, const NRows: usize, const NCols: usize> LayeredKeymap<NRows, NCols> for DefaultKeymap<'a, NRows, NCols> where
{
    fn get_map(&self, activation_matrix: &ActivationMatrix<NRows, NCols>) -> impl IntoIterator<Item = Keyboard> {
        assert!(!self.hardware_break.iter().all(|&x| activation_matrix[x])); // Panic on soft break
        
        let key_map = if activation_matrix[self.function_key] {
            &self.function_keymap
        } else {
            &self.default_keymap
        };
        
        activation_matrix.into_iter().copied().zip(key_map.into_iter().copied()).map(|(z, b)| if z { b } else { Keyboard::NoEventIndicated }).filter(|&key| key != Keyboard::NoEventIndicated)
    }
}
