//!
//! This crate provides the code needed to glom hits into physics ring  items.
//! 
//! 
//! 
use frib_datasource::DataSink;
use rust_ringitem_format::{RingItem, PHYSICS_EVENT};
/// The Glom struct and its implementation are what 
/// do the work.
///  Note that we can add hits and frame boundaries to the
///  event being accumulated.
///  Events are timestamped with the timestamp of the first hit.
/// 
///   hits are stored, internally, as a 16 bit channel/edge number
///  and edge bit and a 64 bit time relative to the start of run.
///  The frame boundary marker is stored as an absolute  frame number with
///  all bits set in the channel/edge word.
///  If the first hit is a frame boundary, it does not start a coincidence interval.
pub struct Glom {
    sink : Box<dyn DataSink>,       // Anything writable.
    sid  : u32,                     // Source id.
    dt   : u64,                     // coincidence interval.
    t0   : Option<u64>,             // when some, the start time of the glom.
    hits : Vec<(u16,u64)>,          // Hits accumulated so far.
}

impl Glom {
    // Start a new hit:
    pub fn new_event(&mut self, chan : u16, time: u64) {
        self.t0 = Some(time);
        self.hits.push((chan, time));
    }
    /// Flush the frame as a ring item. 
    /// Note that this is a no-op if t0 is None (e.g. maybe at end of run?).
    /// t0 will be set to None and hits cleared.
    /// Note that if hits are only frame boundaries, this can lead, at the end run,
    /// dropping them on the floor...why do this? Because we're not sure how to timestamp
    /// frame boundaries.
    pub fn flush(&mut self) {
        if let Some(stamp) = self.t0 {
            let mut item = RingItem::new_with_body_header(
                PHYSICS_EVENT,
                stamp, self.sid, 0
            );
            // Fill the body with hits:
            for (ch, t) in &self.hits {
                item.add(*ch);
                item.add(*t);
            }
            self.sink.write(&item).expect("Unable to write a ring item");
            self.sink.flush();
            self.hits.clear();
            self.t0 = None;
        }
        
    }

    /// Create a glommer, the 
    /// dt and sink are required we set the t0 as none hits as empty.
    ///
    /// ### Parameters:
    /// *   sink - a data sink. The Glom gains ownership.
    /// *   sid  - Source id to put in the ring item body headers.
    /// *   dt   - ticks in coincidence interval.
    /// ### Returns:
    /// a Glom struct.
    /// 
    pub fn new(sink : Box<dyn DataSink>, sid : u32, dt : u64) -> Glom {
        Glom {
            sink : sink,
            sid  : sid,
            dt   : dt,
            t0   : None,            // Not making one.
            hits : Vec::new()
        }
    }
    /// Alter the sid...
    pub fn set_sid(&mut self, sid:  u32) {
        self.sid = sid;
    }
    /// Sometimes we need to just output a ring item.
    /// Since we own the data sink, this allows that:
    ///
    /// ### Parameters:
    /// * item - references a ring itemt to write unaltered.
    /// 
    /// ### Notes:
    /// *  Panics if unable to write.
    /// *  This has no effect on the t0, hits.  At the end of the run, presumably
    ///    one does a flush to write what's there first and then passes the end run item.
    /// 
    pub fn write_item(&mut self, item: &RingItem) {
        self.sink.write(&item).expect("Failed to pass through a ring item");
        self.sink.flush();
    }
    ///
    /// Add a frame boundary to the hits.  This does not
    /// have any effect on the t0 value.
    /// 
    /// ### Parameters:
    /// * fno - absolute frame number.
    /// 
    pub fn add_frame_boundary(&mut self,fno : u64) {
        self.hits.push((0xffff, fno));
    }
    ///
    /// Add a hit.  We construct the channel number word from the channel number
    /// and leading flag.  There are two cases to handle 
    /// 1.  t0 is None. In that case, we are a first hit and set t0 to Some(time). 
    /// and add the channel/time to the hits vector.
    /// 2. t0 is Some, in which case, if we are in the glom interval we just add our hit,
    /// otherwise, flush and start a new event.
    /// 
    /// ###  Parameters
    /// * leading - true if this is a leading edge hit.
    /// * channel - The channel number.
    /// * time    - The absolute time of the hit.
    /// 
    pub fn add_hit(&mut self, leading : bool, channel : u8, time : u64) {
        // Construct the u16 channel/edge tag.

        let chanword : u16 = if leading {
            channel as u16
        } else {
            channel as u16 | 0x8000
        };
        match self.t0 {
            None => self.new_event(chanword, time),
            Some(t0) => {
                if time - t0 <= self.dt {
                    self.hits.push((chanword, time));
                } else {
                    self.flush();
                    self.new_event(chanword, time);
                }
            }
        }
    }
}