<p align="center"><img style="width: 300px;" src="https://github.com/theMackabu/ship/blob/master/.github/assets/ship.png?raw=true" /></p>

Ship is a powerful configuration service that sails your HCL configurations to different formats (JSON, YAML, TOML) while providing a rich fleet of built-in functions for data transformation.

## Features

- **Built-in Functions**

  - **String Operations**:
    - `upper`, `lower`, `trim`, `trimspace`, `trimprefix`, `trimsuffix`
    - String manipulation and formatting
  - **Numeric Operations**:
    - `abs`, `ceil`, `floor`, `max`, `min`, `sum`, `parseint`
    - Mathematical calculations and number parsing
  - **Array/Map Operations**:
    - `join`, `split`, `range`, `merge`, `length`, `unique`, `compact`, `flatten`
    - Collection manipulation and transformation
  - **Cryptographic Functions**:
    - `base64encode/decode`, `urlencode/decode`
    - Multiple hash functions: `md5_hash`, `sha1_hash`, `sha256_hash`, `sha512_hash`
    - UUID generation: `uuid_gen`, `uuidv5`
  - **Date/Time Functions**:
    - `timestamp`, `timeadd`, `parseduration`, `formatdate`
    - Time manipulation and formatting
  - **Network Functions**:
    - `cidrnetmask`, `cidrrange`, `cidrhost`, `cidrsubnets`
    - CIDR calculations and subnet operations
  - **HTTP Client**:
    - `http_get`, `http_post`, `http_json`, `http_put`
    - RESTful API interactions
  - **File Operations**:
    - `file`, `filemd5`, `filesha1`, `filesha256`, `filesha512`
    - File reading and hashing
  - **Vault Integration**:
    - `vault_kv` for HashiCorp Vault key-value store integration

## Configuration

The service is configured via a `config.hcl` file with the following structure:

```hcl
settings {
  listen = "<address:port>"  # Service listen address
  storage = "<path>"         # Storage path for HCL files

  vault {                    # Optional Vault configuration
    url = "<vault-url>"
    token = "<vault-token>"
  }
}
```

## API Usage

> [!CAUTION]
>
> ### Security Notes
>
> - The service should be configured with appropriate access controls
> - Vault token should be kept secure
> - Consider network security when exposing HTTP endpoints
> - File operations are restricted to the configured storage path

### Convert HCL File

```
GET /<path>?lang=<format>
```

Parameters:

- `path`: Path to the HCL file relative to the storage directory
- `lang`: Target format (`json`, `yaml`, `yml`, or `toml`)

The service will:

1. Read the HCL file
2. Process any variables and locals
3. Execute functions
4. Convert to the requested format
5. Return the result as a downloadable file

## Special HCL Blocks

The service supports several special HCL blocks:

- `locals`: For defining local variables
- `let/var/vars`: For variable definitions
- `const`: For constant values that cannot be overridden
- `meta`: For metadata about the configuration

## Error Handling

The service provides detailed error messages in the format:

```
(message)
<error description>

(error)
<status code>
```

## Development

To build and run the service:

1. Ensure you have Rust installed
2. Clone the repository
3. Run `cargo build` to compile
4. Create a `config.hcl` file
5. Run `cargo run` to start the service
