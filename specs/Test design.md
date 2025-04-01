# Test design

Tests skeleton folder will be a monorepo. In the root of the monorepo we will have the following files:


```
- package.json
- .config.toml
- .gitattributes
```


Conten for package.json:

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



For cycle dependecies use 3 of them and put in a cycle dependency.



Now the folder skeleton of tests should be:

```
tests/
|-changes
|--changes_..._test.rs
|--etc
|-workspace
|--workspace..._test.rs
```

Basically each feature in is own folder with the set of tests for that feature, prefixed with module name like example above.



Reminder:

Because we use TempDir dependency the temporary folder should live until the end of the tests. If need for each group the recreation of this skeleton just do it.
