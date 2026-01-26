//! Contains the formatting  stuff for mikumari data
//! 
pub mod mikumari_format {
    // Data type values:

    const TDC_LeadingData : u8 = 0b001011;
    const TDC_TrailingData: u8 = 0b001101;
    const Input_Throttle_T1_Start : u8 = 0b0011001;
    const Input_Throttle_T1_End : u8 = 0b010001;
    const Input_Throttle_T2_Start : u8 = 0b010010;
    const Input_Throttle_T2_End : u8   = 0b010010;
    const Delimeter1  : u8 = 0b011000;
    const Delimeter2  : u8 = 0b011110;

    /// A heartbeat delimieter1 and its data:
    /// 
    struct Delimeter1 {
        delimeter : u64
    }
    struct Delimeter2 {
        delimeter : u64
    }

    // TODO: range check the inputs as they're not full sized.
    impl Delimeter1 {
        pub fn new(time_offset : u16, frame_number: u32) -> Delimeter1 {
            let mut value : u64 = 0;
            value |= (Delimeter1 as u64) << 58;
            value |= (time_offset as u64) << 24;
            value |= frame_number as u64;

            Delimeter1 {
                delimeter : value
            }
        }
        pub fn get(&self) -> u64 {
            self.delimeter
        }
    }
    impl Delimeter2 {
        pub fn new(data_size: u32)-> Delimeter2 {
            let mut value = 0u64;
            value |= (Delimeter2 as u64) << 58;
            let s = data_size as u64;
            value |= (s << 20) | s;

            Delimeter2 {
                delimeter : value
            }

        }
        pub fn get (&self) -> u64 {
            self.delimeter
        }
    }
    #[cfg(test)]
    mod delim1test {
        use super::*;

        #[test]
        fn new_1() {
            let d = Delimeter1::new(0,0);
            assert_eq!(d.get(), 0x6000000000000000)
        }
        #[test]
        fn new_2() {
            let d = Delimeter1::new(0, 1234);
            assert_eq!(d.get() & 0xfffff, 1234u64);
        }
        #[test]
        fn new_3() {
            let d = Delimeter1::new(65535, 0);
            assert_eq!((d.get() >> 24) & 0xffff, 65535);
        }
    }
    #[cfg(test)]
    mod delim2test {
        use super::*;

        #[test]
        fn new_1() {
            let d = Delimeter2::new(0);   // Just the id is set.

            assert_eq!(d.get(), 0x7800000000000000);
        }
        #[test]
        fn new_2() {
            let d = Delimeter2::new(12345);
            assert_eq!(d.get() & 0xfffff, 12345u64);
        }
        #[test]
        fn new_3() {
            let d = Delimeter2::new(12345); 
            assert_eq!((d.get() >> 20) & 0xfffff, 12345u64);
        }
    }
}