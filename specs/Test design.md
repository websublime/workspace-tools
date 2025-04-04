# Test design

Tests skeleton folder will be a monorepo. In the root of the monorepo we will have the following files:


```
- package.json
- .config.toml
- .gitattributes
```


Content for package.json:

```json

{
"name": "root",
"version": "0.0.0",
"workspaces": [
  "packages/package-foo",
  "packages/package-bar",
  "packages/package-baz",
  "packages/package-charlie",
  "packages/package-major",
  "packages/package-tom"
  ]
}
```

Content for .config.toml

```toml
[tools]
git_user_name="bot"
```

Content for .gitattributes

```
* text=auto
```

All of this should have is own functions. Now we need to have functions for defining each package manager files. Managers are:

\- yarn (yarn.lock)
\- pnpm (pnpm-lock.yaml, pnpm-workspace.yaml)
\- bun (bun.lockb)
\- npm (package-lock.json)



Content for pnpm-lock

```yaml
lockfileVersion: '9.0'
```

Content for pnpm-workspace:

```yaml
packages
 - packages/*
```



So that tests can choose which package manager should be defined.

Them monorepo should be inited by git, with branch main using user as 'sublime-bot' and email 'test-bot@websublime.com'. All files added to git and commited with message: 'chore: init monorepo workspace'.

Now let's have a function for each package creation. Each creation will be in his onw branch and in the end merged to main with a tag. Packages will have files. Detailed info for each one of them.



Package foo

* branch feature/package-foo
* folder: package-foo
* files: index.mjs, package.json

Content for package.json

```json
{
    "name": "@scope/package-foo",
    "version": "1.0.0",
    "description": "Awesome package foo",
    "main": "index.mjs",
    "module": "./dist/index.mjs",
    "exports": {
      ".": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      }
    },
    "typesVersions": {
      "*": {
        "index.d.ts": [
          "./dist/index.d.ts"
        ]
      }
    },
    "repository": {
      "url": "git+ssh://git@github.com:websublime/package-foo.git",
      "type": "git"
    },
    "scripts": {
      "test": "echo \"Error: no test specified\" && exit 1",
      "dev": "node index.mjs"
    },
    "dependencies": {
      "@scope/package-bar": "1.0.0"
    },
    "keywords": [],
    "author": "Author",
    "license": "ISC"
}
```

Content for index.mjs:

```javascript
export const foo = "hello foo";
```

Commit message: 'feat: add package foo'

Merge and tag: '@scope/package-foo@1.0.0', 'chore: release package-foo@1.0.0'



Package bar

* branch feature/package-bar
* folder: package-bar
* files: index.mjs, package.json

Content for package.json

```json
{
    "name": "@scope/package-bar",
    "version": "1.0.0",
    "description": "Awesome package bar",
    "main": "index.mjs",
    "module": "./dist/index.mjs",
    "exports": {
      ".": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      }
    },
    "typesVersions": {
      "*": {
        "index.d.ts": [
          "./dist/index.d.ts"
        ]
      }
    },
    "repository": {
      "url": "git+ssh://git@github.com:websublime/package-bar.git",
      "type": "git"
    },
    "scripts": {
      "test": "echo \"Error: no test specified\" && exit 1",
      "dev": "node index.mjs"
    },
    "dependencies": {
      "@scope/package-baz": "1.0.0"
    },
    "keywords": [],
    "author": "Author",
    "license": "ISC"
}
```

Content for index.mjs:

```javascript
import { foo } from 'foo';
export const bar = foo + ":from bar";
```

Commit message: 'feat: add package bar'

Merge and tag: '@scope/package-bar@1.0.0', 'chore: release package-bar@1.0.0'



Package baz

* branch feature/package-baz
* folder: package-baz
* files: index.mjs, package.json

Content for package.json

```json
{
    "name": "@scope/package-baz",
    "version": "1.0.0",
    "description": "Awesome package baz",
    "main": "index.mjs",
    "module": "./dist/index.mjs",
    "exports": {
      ".": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      }
    },
    "typesVersions": {
      "*": {
        "index.d.ts": [
          "./dist/index.d.ts"
        ]
      }
    },
    "repository": {
      "url": "git+ssh://git@github.com:websublime/package-baz.git",
      "type": "git"
    },
    "scripts": {
      "test": "echo \"Error: no test specified\" && exit 1",
      "dev": "node index.mjs"
    },
    "dependencies": {
      "@scope/package-bar": "1.0.0"
    },
    "keywords": [],
    "author": "Author",
    "license": "ISC"
}
```

Content for index.mjs:

```javascript
import { foo } from 'foo';
export const bar = foo + ":from bar";
```

Commit message: 'feat: add package baz'

Merge and tag: '@scope/package-baz@1.0.0', 'chore: release package-baz@1.0.0'



Package charlie

* branch feature/package-charlie
* folder: package-charlie
* files: index.mjs, package.json

Content for package.json

```json
{
    "name": "@scope/package-charlie",
    "version": "1.0.0",
    "description": "Awesome package charlie",
    "main": "index.mjs",
    "module": "./dist/index.mjs",
    "exports": {
      ".": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      }
    },
    "typesVersions": {
      "*": {
        "index.d.ts": [
          "./dist/index.d.ts"
        ]
      }
    },
    "repository": {
      "url": "git+ssh://git@github.com:websublime/package-charlie.git",
      "type": "git"
    },
    "scripts": {
      "test": "echo \"Error: no test specified\" && exit 1",
      "dev": "node index.mjs"
    },
    "dependencies": {
      "@scope/package-foo": "1.0.0"
    },
    "keywords": [],
    "author": "Author",
    "license": "ISC"
}
```

Content for index.mjs:

```javascript
import { foo } from 'foo';
export const bar = foo + ":from bar";
```

Commit message: 'feat: add package charlie'

Merge and tag: '@scope/package-charlie@1.0.0', 'chore: release package-charlie@1.0.0'



Package major

* branch feature/package-major
* folder: package-major
* files: index.mjs, package.json

Content for package.json

```json
{
    "name": "@scope/package-major",
    "version": "1.0.0",
    "description": "Awesome package major",
    "main": "index.mjs",
    "module": "./dist/index.mjs",
    "exports": {
      ".": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      }
    },
    "typesVersions": {
      "*": {
        "index.d.ts": [
          "./dist/index.d.ts"
        ]
      }
    },
    "repository": {
      "url": "git+ssh://git@github.com:websublime/package-major.git",
      "type": "git"
    },
    "scripts": {
      "test": "echo \"Error: no test specified\" && exit 1",
      "dev": "node index.mjs"
    },
    "dependencies": {
      "@websublime/pulseio-core": "^0.4.0",
      "@websublime/pulseio-style": "^1.0.0",
      "lit": "^3.0.0",
      "rollup-plugin-postcss-lit": "^2.1.0"
    },
    "keywords": [],
    "author": "Author",
    "license": "ISC"
}
```

Content for index.mjs:

```javascript
import { foo } from 'foo';
export const bar = foo + ":from bar";
```

Commit message: 'feat: add package major'

Merge and tag: '@scope/package-major@1.0.0', 'chore: release package-major@1.0.0'



Package tom

* branch feature/package-tom
* folder: package-tom
* files: index.mjs, package.json

Content for package.json

```json
{
    "name": "@scope/package-tom",
    "version": "1.0.0",
    "description": "Awesome package tom",
    "main": "index.mjs",
    "module": "./dist/index.mjs",
    "exports": {
      ".": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      }
    },
    "typesVersions": {
      "*": {
        "index.d.ts": [
          "./dist/index.d.ts"
        ]
      }
    },
    "repository": {
      "url": "git+ssh://git@github.com:websublime/package-tom.git",
      "type": "git"
    },
    "scripts": {
      "test": "echo \"Error: no test specified\" && exit 1",
      "dev": "node index.mjs"
    },
    "dependencies": {
      "@scope/package-bar": "1.0.0",
      "open-props": "^1.6.19",
      "postcss": "^8.4.35",
      "postcss-cli": "^11.0.0",
      "postcss-custom-media": "^10.0.3",
      "postcss-import": "^16.0.1",
      "postcss-jit-props": "^1.0.14",
      "postcss-mixins": "^9.0.4",
      "postcss-nested": "^6.0.1",
      "postcss-preset-env": "^9.4.0",
      "postcss-simple-vars": "^7.0.1",
      "typescript": "^5.3.3",
      "vite": "^5.1.4"
    },
    "keywords": [],
    "author": "Author",
    "license": "ISC"
}
```

Content for index.mjs:

```javascript
import { foo } from 'foo';
export const bar = foo + ":from bar";
```

Commit message: 'feat: add package major'

Merge and tag: '@scope/package-major@1.0.0', 'chore: release package-major@1.0.0'



For cycle dependencies use 3 of them and put in a cycle dependency function fixture.

Mandatory:

Use rstest crate to create all the fixtures functionality, so that i can reuse them in the real tests that we will generate.

Now the folder skeleton of tests should be:

```
tests/
|-fixtures
|--mod.rs
|--... (fixtures files)
|-changes_..._test.rs
|-workspace..._test.rs
|-versioning..._test.rs
|-utils..._test.rs
|-tasks..._test.rs
...
```

Basically each feature is prefixed with module name like example above, fixtures should exist in fixtures folders and have a common for monorepo structure and them if needed fixtures for each module by file. We can use rstest for generating fixtures.



Reminder:

Because we use TempDir dependency the temporary folder should live until the end of the tests. If need for each group the recreation of this skeleton just do it.

Request to generate tests:

Based in the test design spec i would like to generate tests for the changes module. I will need a detail implementation for each step, files location, follow clippy rules and cover all apis exported by the module. I think the best is to not produce everything in one shot but step by step. Let's start by the fixtures then in each prompt we generate each feature. Put the list of steps here so that i can tell you to proceed to the next step. I will provide for your context also all the available apis for the crates:
- sublime_git_tools
- sublime_standard_tools
- sublime_package_tools
- sublime_monorepo_tools

Do not make assumptions of anything in those crates, follow the api spec for your usage, meaning do not invent methods that are not specified in the doc. Packages names to use in all tests should be always the same as defined in this doc.