<div align="center">

# Task for Sofixit Backend Developer
**This repository houses my solution for a couple of tasks from Sofixit**

</div>
</br>

## The tasks
Three services:
1. Service which returns at **/generate/json/{size}** a list of jsons with the indicated size and structured below with random values.
```json
{
   "_type": "Position",
   "_id": 65483214,
   "key": null,
   "name": "Oksywska",
   "fullName": "Oksywska, Poland",
   "iata_airport_code": null,
   "type": "location",
   "country": "Poland",
   "geo_position": {
      "latitude": 51.0855422,
      "longitude": 16.9987442
   },
   "location_id": 756423,
   "inEurope": true,
   "countryCode": "PL",
   "coreCountry": true,
}
```
2. Service that takes the data from the first one and converts it to csv.
    - The first endpoint should return the retrieved data in the format 'type, _id, name, type, latitude, longitude'.
    - The second endpoint that always returns the retrieved data in the given csv structure, i.e. we specify in the query 'id, latitude, longitude' and we expect it to return (from data above) '65483214, 51.0855422, 16.9987442'.
    - A third endpoint which expects the input to define simple mathematical operations in the form of a list, e.g. 'latitude*longitude,sqrt(location_id)' and is expected to return (from data above) '3.0052538,869.7258188'.

3. A service that performs queries on the second one and displays simple performance reports. The report should include information such as CPU usage, memory usage over time for each of the previous services and http query time between services 3->2->1. Report on 1k,10k,100k jsons generated.

## My assumptions
Since the tasks are about implementing of multiple services, I assumed that they should be separate i.e. they can't access each other's internal state and can only communicate with public API, but for the sake of simplicity I put all of them behind one HTTP server.

This type of very open task can be very extensive in terms of implementation, so I decided to focus on the core functionality and the things that I would do in the future if i were to continue working on this project, I described in the "What could be done in the future" sections.

## Task 1, the JSON generator
This one was fairly simple because of the fact, that the data needs to be truly random. If we had to pick a random place and then get all of the data from it, then generating 100k of those would be a pain and would require some API calls for a map service. Fortunately i could just build a simple random generator for the data. I defined a structure with desired fields and used derive functionality of Rust's Serde crate to automatically allow for JSON serialization. Then i just had to define how to generate fields randomly. For that i used a mix of random number generation and providing a list of possible values for some fields. I also made the generator parallel, so that it can generate 100k of those in a reasonable time.

Usage: **/generate/json/{size}?perf={true|false}**
- size: number of JSONs to generate
- perf: whether to measure performance or not (more on that later)


### What could be done in the future:
- Improve parallel generation, since current implementation uses fully automatic solution, which is simple but is not squeezing the most out of the hardware (which can be seen in the reports from task 3)
- Improve error codes and messages, since currently they are not very descriptive
- Add simulation of a delay of getting the data from a database or an API, to simulate a real world scenario

## Task 2, the CSV converter
In this task each next subpoint is a superset of the previous one, so if the last one was working well, all previous ones can use the same code to reduce the code complexity.

The first subtask just needs to return a subset of the data. My first implementation was just to create another structure and define how it will be obtained from the main one. Then again serialize it with Serde.

The second subtask required a bit more work. I used regular expressions to validate the query and array filtering to get only the desired fields.

The third subtask was the most interesting one since I had to create my own expression language for evaluation of the expressions. I used a Rust crate called [pest](https://pest.rs/) to create my grammar and used inbuilt pratt parser to parse the expression into an abstract syntax tree. Then I defined some rules for evaluation.

My language supporrts:
- Basic arithmetic operations: +, -, *, /
- Integer number input: 456, -89
- String input: "abc", "def"
- Field access: latitude, longitude, location_id etc.
- Math functions: sqrt, pow2
- String operations: string + string, string * number
- Parentheses: (1 + 2) * 3
- Unary minus: -(...)

Usage: **/generate/csv/{size}?perf={true|false}**
- size: number of CSV rows to generate
- perf: whether to measure performance or not (more on that later)

### What could be done in the future:
- Extend the language to support more features (functions, boolean algebra, more intelligent type conversions)
- Make errors more indicative where the issue happened
- Use better implementation of pest (like [faster-pest](https://github.com/Mubelotix/faster-pest) that claims 700% performance increase on example JSON parsing benchmark. The crate doesn't ship with pratt parser though, so it would have to be implemented manually)
- Write more tests

## Task 3, the performance report
This task was a challenging one, but not because of the complexity of the task itself, but the fact that Rust is not made for performance measuring internally. It would be way better to use some external tool for that. For my (keep in mind, very makeshift) solution I used [sysinfo](https://docs.rs/sysinfo/latest/sysinfo/) crate. It uses platform specific system calls to get wanted statistics. Then I modified each endpoint to support a query parameter that would enable performance measuring. I also added a new endpoint that would return a simple plaintext report.
While generating the report i encoutered a problem with the fact that sometimes CPU usage was reported as 0.0%. It is because sysinfo sometimes will not update the data before reading the values.

Usage: **/measure/csv/{size}**
- size: size of request to measure performance for

### What could be done in the future:
- If the project was forced to use internal measuring, the overhead of measuring should be reduced to provide more accurate data
- Use some external tool for performance measuring
- Detatch the perrformance measuring from internal logic of the services
- Enable multi-sampling for more accurate results

## The reports
My structure for the reports is as follows:
```
STATISTICS FOR CALLING /generate/csv/{length}:
- CPU utilization: []
- Memory utilization: []
- Time elapsed: {}

STATISTICS FOR CALLING /generate/json/{length} from /generate/csv/{length}:
- CPU utilization: []
- Memory utilization: []
- Time elapsed: {}

All utilization values are measured with 200ms interval.
```
I will provide reports for 1k, 10k and 100k of generated JSONs and direct time measurements from [httpstat](https://github.com/reorx/httpstat), to show the overhead of the measurement. My machine includes a Ryzen 5800X CPU and 16GB of RAM.

### 1k JSONs
```
STATISTICS FOR CALLING /generate/csv/1000:
- CPU utilization: [0.0, 7.1428576]
- Memory utilization: [25538560, 29335552]
- Time elapsed: 1369 ms

STATISTICS FOR CALLING /generate/json/1000 from /generate/csv/1000:
- CPU utilization: [0.0]
- Memory utilization: [28827648]
- Time elapsed: 245 ms

All utilization values are measured with 200ms interval.
```

Calling /generate/json/1000 directly from httpstat:
- Server processing: 1ms
- Content transfer: 1ms

Calling /generate/csv/1000 directly from httpstat:
- Server processing: 6ms
- Content transfer: 1ms

### 10k JSONs
```
STATISTICS FOR CALLING /generate/csv/10000:
- CPU utilization: [0.0, 0.0]
- Memory utilization: [28344320, 34443264]
- Time elapsed: 450 ms

STATISTICS FOR CALLING /generate/json/10000 from /generate/csv/10000:
- CPU utilization: [0.0]
- Memory utilization: [29577216]
- Time elapsed: 247 ms

All utilization values are measured with 200ms interval.
```

Calling /generate/json/10000 directly from httpstat:
- Server processing: 8ms
- Content transfer: 3ms

Calling /generate/csv/10000 directly from httpstat:
- Server processing: 44ms
- Content transfer: 1ms

### 100k JSONs
```
STATISTICS FOR CALLING /generate/csv/100000:
- CPU utilization: [0.0, 0.0]
- Memory utilization: [28872704, 78110720]
- Time elapsed: 691 ms

STATISTICS FOR CALLING /generate/json/100000 from /generate/csv/100000:
- CPU utilization: [0.0]
- Memory utilization: [31703040]
- Time elapsed: 314 ms

All utilization values are measured with 200ms interval.
```

Calling /generate/json/100000 directly from httpstat:
- Server processing: 79ms
- Content transfer: 39ms

Calling /generate/csv/100000 directly from httpstat:
- Server processing: 416ms
- Content transfer: 5ms
