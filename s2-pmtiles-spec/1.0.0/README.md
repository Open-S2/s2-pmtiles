# Open Vector Tile Specification 1.0.0

TODO:

[ ] - Overview
[ ] - A fixed-size 127-byte header
[ ] - A root directory

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT",
"SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in
this document are to be interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

## 1. Purpose

This document specifies a space-efficient encoding format for tiled geographic vector data. It is designed to be used in browsers or server-side applications for fast rendering or lookups of feature data.

## 2. File Format

The Vector Tile format primarily uses [Google Protocol Buffers](https://developers.google.com/protocol-buffers/) as a encoding format. Protocol Buffers are a language-neutral, platform-neutral extensible mechanism for serializing structured data. Some

### 2.1. File Extension

The filename extension for Vector Tile files SHOULD be `s2pm`. For example, a file might be named `data.s2pm`.

### 2.2. Multipurpose Internet Mail Extensions (MIME)

When serving Vector Tiles the MIME type SHOULD be `application/vnd.s2pm`.

## 3. Overview
