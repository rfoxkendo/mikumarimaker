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

    impl Delimeter1 {
        pub fn new(time_offset : u16, frame_number: u32) -> Delimeter1 {
            let mut value : u64 = 0;
            value |= (Delimeter1 as u64) << (32-6);
            value |= (time_offset as u64) << 24;
            value |= frame_number as u64;

            Delimeter1 {
                delimeter : value
            }
        }
    }
}