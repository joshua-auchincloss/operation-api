# use

Declares a namespace to be used. If in a namespace, the use is private. If in a `lib.pld`, the use is public.

Syntax:

```pld
// foo.pld

namespace foo;

#![version(1)]

use schema::bar;
use schema::bar::SomeObject;

use external_ns::baz;
```

```pld
// lib.pld

namespace abc_corp;

use foo;
use bar;
```

## Multiple Items

You can import multiple items from a namespace using brace syntax:

```pld
use schema::bar::{
	SomeObject,
	AnotherObject,
}
```

Single items can be written with or without braces (but will be formatted without):

```pld
use schema::bar::SomeObject;
use schema::bar::{SomeObject};
```
