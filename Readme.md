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

Defenestration means to throw someone out a window.  In the context of FRIB/NSCLDAQ, it means to take windowed (frame files) and turn them into something 'else'.  For time data,
the framing is sort of maintained, but the time data are transformed into a format that
is no longer mikumari dependent.

The program passes all ring items that are not Mikumari time frames
(type 51) through without 
modification.

The program can accept data from an input file, stdin, or ringbuffer and write data to an output file or stdout.  Future work may allow this to send output to an online ringbuffer.

The output of this program are a sequence of ring items.

*  The type of the ring items is ```PHYSICS_EVENT```
*  The ring items have body headers with timestamps that are the same as the timestamps in the input ring items.


Body contents are:

|     Contents     |   Size    | Notes   |
|------------------|-----------|---------|
| Absolute frame number | uint64_t | Only the bottom 16 bits are nonzero |
| Absolute hit     | uint16_t, uint64_t | See below |
|   ...            | ...                | ... |


Absolute hits  are a 16 bit word followed by a 64 bit word:

|  Contents       | Size     | Notes     |
|-----------------|----------|-----------|
| channel and edge| uint16_t | The top bit is set for falling edge, the remainder of the word is the channel number.
| Absolute time   | uint64_t | Time of the hit relative to the start of the run. |

Note the absolute time is computed fromt he mikumari hit time and the timestamp of the ring item.  It will roll over after over 200 days and the LSB as for the ring item timestamp is 0.9765625pico seconds.


Usage:
```
defenestrator source-uri out-uri
```

Where:
|  parameter | Meaning                    |
|------------|----------------------------|
| source-uri | Is a the data source URI as per standard FRIB/NSCLDAQ URI format |
| sink-uri | is a URI specifying either the file or or ring buffer to which data are written |

source and sink URIS  can have the form:

* file:///absolute-path-to-some-file for  file data.
* tcp://hostname/ringname for ringbuffers. Note that output-uris must use ```localhost`` for the hostname
*  file://- is a special path that for sourcde_uri's means stdin and for sink-uris stdout.



