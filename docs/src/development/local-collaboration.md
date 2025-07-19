# Local Collaboration

First, make sure you've installed CodeOrbit's backend dependencies for your platform:

- [macOS](./macos.md#backend-dependencies)
- [Linux](./linux.md#backend-dependencies)
- [Windows](./windows.md#backend-dependencies)

Note that `collab` can be compiled only with MSVC toolchain on Windows

## Database setup

Before you can run the `collab` server locally, you'll need to set up a `CodeOrbit` Postgres database.

### On macOS and Linux

```sh
script/bootstrap
```

This script will set up the `CodeOrbit` Postgres database, and populate it with some users. It requires internet access, because it fetches some users from the GitHub API.

The script will seed the database with various content defined by:

```sh
cat crates/collab/seed.default.json
```

To use a different set of admin users, you can create your own version of that json file and export the `SEED_PATH` environment variable. Note that the usernames listed in the admins list currently must correspond to valid GitHub users.

```json
{
  "admins": ["admin1", "admin2"],
  "channels": ["CodeOrbit"]
}
```

### On Windows

```powershell
.\script\bootstrap.ps1
```

## Testing collaborative features locally

### On macOS and Linux

Ensure that Postgres is configured and running, then run CodeOrbit's collaboration server and the `livekit` dev server:

```sh
foreman start
# OR
docker compose up
```

Alternatively, if you're not testing voice and screenshare, you can just run `collab`, and not the `livekit` dev server:

```sh
cargo run -p collab -- serve all
```

In a new terminal, run two or more instances of CodeOrbit.

```sh
script/CodeOrbit-local -3
```

This script starts one to four instances of CodeOrbit, depending on the `-2`, `-3` or `-4` flags. Each instance will be connected to the local `collab` server, signed in as a different user from `.admins.json` or `.admins.default.json`.

### On Windows

Since `foreman` is not available on Windows, you can run the following commands in separate terminals:

```powershell
cargo run --package=collab -- serve all
```

If you have added the `livekit-server` binary to your `PATH`, you can run:

```powershell
livekit-server --dev
```

Otherwise,

```powershell
.\path\to\livekit-serve.exe --dev
```

In a new terminal, run two or more instances of CodeOrbit.

```powershell
node .\script\CodeOrbit-local -2
```

Note that this requires `node.exe` to be in your `PATH`.

## Running a local collab server

If you want to run your own version of the CodeOrbit collaboration service, you can, but note that this is still under development, and there is no good support for authentication nor extensions.

Configuration is done through environment variables. By default it will read the configuration from [`.env.toml`](https://github.com/codeorbit-industries/CodeOrbit/blob/main/crates/collab/.env.toml) and you should use that as a guide for setting this up.

By default CodeOrbit assumes that the DATABASE_URL is a Postgres database, but you can make it use Sqlite by compiling with `--features sqlite` and using a sqlite DATABASE_URL with `?mode=rwc`.

To authenticate you must first configure the server by creating a seed.json file that contains at a minimum your github handle. This will be used to create the user on demand.

```json
{
  "admins": ["nathansobo"]
}
```

By default the collab server will seed the database when first creating it, but if you want to add more users you can explicitly reseed them with `SEED_PATH=./seed.json cargo run -p collab seed`

Then when running the CodeOrbit client you must specify two environment variables, `CODEORBIT_ADMIN_API_TOKEN` (which should match the value of `API_TOKEN` in .env.toml) and `CODEORBIT_IMPERSONATE` (which should match one of the users in your seed.json)
