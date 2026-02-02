error[E0107]: method takes 0 generic arguments but 1 generic argument was supplied
   --> src/geoip.rs:76:18
    |
 76 |     match reader.lookup::<City>(ip_addr) {
    |                  ^^^^^^-------- help: remove the unnecessary generics
    |                  |
    |                  expected 0 generic arguments
    |
note: method defined here, with 0 generic parameters
   --> /home/swordreforge/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/maxminddb-0.27.1/src/reader.rs:194:12
    |
194 |     pub fn lookup(&'de self, address: IpAddr) -> Result<LookupResult...
    |            ^^^^^^

error[E0609]: no field `country` on type `LookupResult<'_, Mmap>`
  --> src/geoip.rs:79:55
   |
79 |             let country = if let Some(country) = city.country {
   |                                                       ^^^^^^^ unknown field

error[E0282]: type annotations needed
  --> src/geoip.rs:81:21
   |
81 |                     names.get("zh-CN")
   |                     ^^^^^ cannot infer type

error[E0282]: type annotations needed
  --> src/geoip.rs:83:31
   |
83 |                         .map(|s| s.to_string())
   |                               ^  - type must be known at this point
   |
help: consider giving this closure parameter an explicit type
   |
83 |                         .map(|s: /* Type */| s.to_string())
   |                                ++++++++++++

error[E0609]: no field `city` on type `LookupResult<'_, Mmap>`
  --> src/geoip.rs:93:59
   |
93 |             let city_name = if let Some(city_data) = city.city {
   |                                                           ^^^^ unknown field

error[E0282]: type annotations needed
  --> src/geoip.rs:95:21
   |
95 |                     names.get("zh-CN")
   |                     ^^^^^ cannot infer type

error[E0282]: type annotations needed
  --> src/geoip.rs:97:31
   |
97 |                         .map(|s| s.to_string())
   |                               ^  - type must be known at this point
   |
help: consider giving this closure parameter an explicit type
   |
97 |                         .map(|s: /* Type */| s.to_string())
   |                                ++++++++++++

error[E0609]: no field `subdivisions` on type `LookupResult<'_, Mmap>`
   --> src/geoip.rs:107:59
    |
107 | ...bdivisions) = city.subdivisions {
    |                       ^^^^^^^^^^^^ unknown field

error[E0282]: type annotations needed
   --> src/geoip.rs:108:36
    |
108 |                 if let Some(sub) = subdivisions.into_iter().next() {
    |                                    ^^^^^^^^^^^^ cannot infer type

error[E0282]: type annotations needed
   --> src/geoip.rs:110:25
    |
110 |                         names.get("zh-CN")
    |                         ^^^^^ cannot infer type

error[E0282]: type annotations needed
   --> src/geoip.rs:112:35
    |
112 | ...                   .map(|s| s.to_string())
    |                             ^  - type must be known at this point
    |
help: consider giving this closure parameter an explicit type
    |
112 |                             .map(|s: /* Type */| s.to_string())
    |                                    ++++++++++++

Some errors have detailed explanations: E0107, E0282, E0609.
For more information about an error, try `rustc --explain E0107`.
error: could not compile `rustblog` (bin "rustblog") due to 11 previous errors
