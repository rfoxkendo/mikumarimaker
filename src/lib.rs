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
    const Delimeter1  : u8 = 0b011100;
    const Delimeter2  : u8 = 0b011110;

    /// A heartbeat delimieter1 and its data:
    /// 
    struct Delimeter1 {
        delimeter : u64
    }
    struct Delimeter2 {
        delimeter : u64
    }
    // Sample does not show the throttles so skip to the chase:
    // Not sure how software tells the difference between high and 
    // low resolution as I don't see separate data types for them.
    // Assumption:  Time over threshold will only be present in the trailing
    // time as the TOT is from leading to trailing edge(?).
    struct HRTDCLeading {
        leading : u64
    }
    struct HRTDCTrailing {
        trailing : u64
    }

    struct LRTDCLeading {
        leading : u64
    }
    struct LRTDCTrailing {
        trailing : u64
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
    impl HRTDCLeading {
        pub fn new(chan : u8, tot : u32, time : u32) -> HRTDCLeading {
            let mut value = (TDC_LeadingData as u64) << 58;
            value |= (chan as u64) << 51;
            value |= (tot as u64)     << 29;
            value |= time as u64;

            HRTDCLeading {
                leading : value
            }
        }
        // Getters:

        pub fn channel(&self) -> u8 {
            ((self.leading  >> 51) & 0x7f) as u8
        }
        pub fn TOT(&self) -> u32 {
            ((self.leading >> 29) & 0x3fffff) as u32
        }
        pub fn Time(&self) -> u32 {
            (self.leading & 0x1FFFFFFF) as u32
        }
    }
    // In fact, other than the data type fie.d, 
    // this is just like the leading edge so we do do some dirty stuff.

    impl HRTDCTrailing {
        pub fn new(chan : u8, tot : u32, time : u32) -> HRTDCTrailing {
            let leading = HRTDCLeading::new(chan, tot, time);
            // maks off the data type and replace it with 0x34

            let mut data = leading.leading;
            data &= !(TDC_LeadingData as u64) << 58;
            data |= (TDC_TrailingData as u64)<< 58;

            HRTDCTrailing {
                trailing : data
            }
        }
        // Here's where the dirt is, we use the LE functions.
        // The dirt we use in this method gets used in all others as well.
        pub fn channel(&self) -> u8 {
            let leading = HRTDCLeading { leading: self.trailing};  // the dirt:

            leading.channel()
        }
        pub fn TOT(&self) -> u32 {
            let leading = HRTDCLeading { leading: self.trailing};
            leading.TOT()
        }
        pub fn Time(&self) -> u32 {
            let leading = HRTDCLeading { leading: self.trailing};
            leading.Time()
        }
    }
    #[cfg(test)]
    mod delim1test {
        use super::*;

        #[test]
        fn new_1() {
            let d = Delimeter1::new(0,0);
            assert_eq!(d.get(), 0x7000000000000000)
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