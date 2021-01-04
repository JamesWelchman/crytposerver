# CryptoServer

CryptoServer provides a technique for deligating cryptographic
hmacing to a server. The author feels this has advantages for
many applications.

It is specifically designed for when a secret is non-rotatable
after creation. We solve this problem by using a very large
array of secrets. Even if millions of requests are processed,
the probability of using the same secret twice is close to 0.

[Usage](#usage)
[Building](#building)
[Implementation Notes](#implementation)
[Runtime Parameters](#runtime-parameters)
[Tips and Tricks](#tips-and-tricks)


## <a name="usage"></a> Usage

To demonstrate this we will start with standard python code
and refactor it to use the cryptoserver.

```python
import hmac
from hashlib import sha256


# A secret key, hardcoded or stored in AWS secrets
# or read from the file system etc.
SECRET_KEY = bytes.fromhex(
    "13dd9f2fed7d8c61a1782d450ee3505e"
)


def compute(msg):
	"""
	compute will compute the hmac of the given msg
	msg should be of type bytes.
	"""
	hasher = hmac.new(key=SECRET_KEY, digestmod=sha256)
	hasher.update(msg)
	return hasher.digest()


if __name__ == '__main__':
	# Compute some digests
	print(compute(b"hello world"))
	print(compute(b"apples and pears")
```

Now if we refactor this code to use the cryptoserver.
We make use the third party requests library.

```python
from requests import post


CRYPTO_SERVER_URL = "http://localhost:8080/hmac";


def compute(msg):
	res = post(CRYPTO_SERVER_URL, data=msg)
	if res.status_code != 200:
		raise RuntimeError("couldn't hmac data")

	return res.content


if __name__ == '__main__':
	# Compute some digests
	print(compute(b"hello world"))
	print(compute(b"apples and pears")
```

We see our hmac call and the required secrets are deligated
to the cryptoserver.


## <a name="building"></a> Building

This requires the Rust programming language to be installed.
Following this.

```bash
   $ cargo build --release
   $ cp target/release/crytposerver ${HOME}/.local/bin
```

NOTE: Any directory in $PATH may be used, below the author
assumes cryptoserver is installed into one of the directories.


## <a name="implementation"></a> Implementation

The default behaviour of cryptoserver is to read a single secret
from the filesystem. This is expected to be a single file of length
32 at the position /secrets/secret.

### Creating the default runtime

To run on the default with a single secret follow these steps:

```bash
   $ mkdir /secrets
   $ dd if=/dev/urandom of=/secrets/secret bs=32 count=1
   $ cryptoserver
```

When running this way, cryptoserver simply hmac's the data it
is sent as one would expect. There is no special behaviour.

### Mode16 and Mode32

The biggest feature of cryptoserver is the ability to use a large
number of secrets. These are set with the environment variable
CRYPTOSERVER_MODE.

To run in Mode16 we need to generate (2 ^ 16 = 65536) secrets.

```bash
   $ mkdir /secrets
   $ dd if=/dev/urandom of=/secrets/0000 bs=32 count=65536
   $ CRYPTOSERVER_MODE=MODE16 cryptoserver
```

When running in Mode16, request data is hashed down to a 16 bit
unsigned integer. This is _how_ cryptoserver decides which secret
to use for the hmacing.

#### Mode32

Mode32 extends Mode16 to use (2 ^ 32 = 4294967296) secrets.
We make use of python to permutate over all possible 4 char
hexstrings.

NOTE: If running on AWS etc. this requires approx 160GB of HDD
space.

```bash
   $ mkdir /secrets
   $ python
   >>> HEXCHARS = "0123456789abcdef"
   >>> from os import urandom
   >>> from itertools import product
   >>> def hexchars():
   ...     for a in HEXCHARS:
   ...         for b in HEXCHARS:
   ...             yield a + b
   ...
   >>> for name in product(hexchars(), hexchars()):
   ...     name = "".join(name)
   ...     with open ("/secrets/{!s}".format(name), 'wb') as file:
   ...         file.write(urandom(32 * 65536))
   ...
   >>>
   $ CRYPTOSERVER_MODE=Mode32 cryptoserver
```

2 ^ 32 is a huge number of secrets and will almost certainly mean
you never need to rotate the secrets unless the HDD which is storing
them is somehow compromised.


## <a name="runtime-parameters"></a> Runtime Parameters

All runtime parameters are set with environment variables.

### CRYPTOSERVER_MODE

This can be one of three values: Mode0, Mode16, Mode32.
The default is Mode0 (see Implementation section above).

### CRYPTOSERVER_SECRETDIR

The directory to search for the secret(s).
The application will default to /secrets.

### RUST_LOG

Set the logging level, NOTE: by default there will be no logging
output.

The lowest level of logging of this application is DEBUG.


### CRYPTOSERVER_BIND

The bind path given to the server.
The default is "0.0.0.0:8080".


```bash
   $ RUST_LOG=INFO cryptoserver
   2021-01-03T11:18:58Z INFO  cryptoserver cryptoserver started
```

## <a name="tips-and-tricks"></a> Tips and Tricks 

If one sets up an EC2 instance with a HDD. After generating the secrets,
one can "lose" the SSH key - this keeping the secrets secret forever!
