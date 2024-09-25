# Gravitron ECS
Performant ECS for Gravitron
## Features
- Components with a derive macro
- Systems as normal functions with queries to query the world, commands to modify entities and global resources
- Ability for parallel execution with automatic detection for interference between systems and parallelizing optimization using [RLF](https://en.wikipedia.org/wiki/Recursive_largest_first_algorithm) 
## Benchmarks
format: debug release
### create entity
initial: 23ys 4ys
average: 1.6ys 200ns
### add component
initial: 11ys 2ys
average: 1.2ys 140ns
### get component
initial: 1.8ys 200ns
average: 750ns 80ns
