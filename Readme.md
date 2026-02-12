## Products of this repository:

Two binaries here:
*  mikumarimaker takes a raw mikumari time data file and makes a ring item file.
*  defenestrator takes the output of e.g. mikumarimaker and output defenestrated ring items.

###  mikumarimaker

This program takes raw mikumari High precision TDC frame files and produces ring item frame files.
These have the following characteristics:

*  The data prior to the first heartbeat are discarded.
*  The item type is 51  - time frames.
*  The timestamp in the body header is the computed timestamp of the heartbeat that starts the frame.  By computed timestamp I mean that the first frame number is assigned a timestamp of 0. Subsequent frames have a timestamp that is the frame number * the number of tdc ticks per frame.  This assumes that (as documented):
    - frames are 524.288&mu;seconds apart.
    - The TDC tick (LSB resolution) is 0.9765625pico seconds.

Note: the frame number relative to the start of data are internally maintained as a uint64_t.  Note the 64 bit timestamp will rollover after over 200 days.

The ring item body contains the following:

|  Contents      | Size    |  Notes | 
|----------------|---------|--------|
| Raw frame number | uint64_t | only the least significant bits are meaningful. |
| Raw hit        | uint16_t   | The raw TDC value (rising or falling edges). |
|    ...         |   ...      | ...|

Where there will be as many raw hit values as there are up to the next heartbeat.
The following are filtered out:
* Delimeter 1 since the information it has is implicit in the ring item size.
* Throttle words.  Note that these could easily be added back if desired.

Usage of the program:

```
mikumarimaker [options] infile outuri
```
* options control the begin/end run items that bracket the data.  See OPTIONS 
below for what they are and the default values. 
* infile is the path to the file that contains raw mikumari data.
* outuri is the URI of the output this can be a file: or tcp://localhost/ring_name
for online data.


#### OPTIONS

The program accepts several command line options to control what goes in the begin run
and end run items that bracket the data:

| Long form |  Short form | Default | Meaning of value|
|-----------|-------------|---------|-----------------|
| --title   | -t          | ```"No title set"``` | Title string in begin and end run items |
| --run     | -r          | ```0```       | Run number in begin and end run items |
| --source-id | -s        | ```0```       | Source id put in body headers. |
| --version | -v          | N/A     | outputs the program version and exits |
| --help    | -h          | N/A     | outputs brief program usage help and exits |



### defenestrator

Defenestration means to throw someone out a window.  In the context of FRIB/NSCLDAQ, it means to take windowed (frame files) and turn them into 'something else'.  For time data, that 'something else' is hits accumulated into events based on a settable coincidence interval.  Since
frame times are not considered to be precise, the position of the frames in the data are retained.

The program passes all ring items that are not Mikumari time frames
(type 51) through without modification.

The program can accept data from an input file, stdin, or ringbuffer and write data to an output file or stdout.  Future work may allow this to send output to an online ringbuffer.

The output of this program are a sequence of ring items.

*  The type of the ring items is ```PHYSICS_EVENT```
*  The ring items have body headers
*  Each PHYSICS_EVENT ring item is a set of hits that occured within a settable coincidence interval
*  event ring items have body headers:
    * The timestamps on the body headers are the time of the first hit.
    * The source id on the body headers is the source id of the input data.
*  A special hit channel identifies where frame boundaries are.

The defenestrator outputs what it thinks are events given a coincidence
interval.  Each event will have a timestamp derived from the first hit in the event.  Hits consist of a 16 bit channel/edge word followed by a 64 bit absolute time word:


|  Contents       | Size     | Notes     |
|-----------------|----------|-----------|
| channel and edge| uint16_t | The top bit is set for falling edge, the remainder of the word is the channel number.
| Absolute time   | uint64_t | Time of the hit relative to the start of the run. |
| TOT             | uint32_t | Time over threshold |

Frame boundaries are shown by a hit with the channel and edge field, and TOT set to 0xffff.  The "_time_" of that hit is the absolute frame number. For example:

```
0xffff
0x0000000000001000
0xffff
```

is a frame boundary with the absolute frame number 4096.  Note that the data are little endian so for this example in 16bit words in the dumper will be in the following order:

```
0xffff   - Frame boundary flag.
0x1000  \   Least significant 32 bits
0x0000  /   of the frame number
0x0000  \   Most significant 32  bits
0x0000  /   of the frame number.
0xffff  -   fake time over threshold field.
```

Note the absolute times of actual hits are computed from the mikumari hit time and the timestamp of the input ring item that contained them (see mikumarimaker).  It will roll over after over 200 days and the LSB as for the ring item timestamp is 0.9765625pico-seconds.

The timestamp of the input ring items (from mikumarimaker) are the computed time, after the first frame of the frame. For example times in the 0'th frame will not be altered, while times in the second frame will have 
```
524.288 usec/frame* 10^6 ps/usec / 0.9765625 ps/tdc-tick
```
added to them. etc.

Usage:
```
defenestrator --dt coincidence-window source-uri out-uri
```

Where:
|  parameter | Meaning                    |
|------------|----------------------------|
| source-uri | Is a the data source URI as per standard FRIB/NSCLDAQ URI format |
| sink-uri | is a URI specifying either the file or or ring buffer to which data are written |
| --dt     | The argument of this option is the coincidence window in TDC Ticks |

source and sink URIS  can have the form:

* file:///absolute-path-to-some-file for  file data.
* tcp://hostname/ringname for ringbuffers. Note that output-uris must use ```localhost`` for the hostname
*  file://- is a special path that for source_uri's means stdin and for sink-uris stdout.



