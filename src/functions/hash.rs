declare_fns!(default.ctx,
    FN_BCRYPT => name!("hash", "bcrypt"),
    FN_MD5 => name!("hash", "md5"),
    FN_SHA1 => name!("hash", "sha1"),
    FN_SHA256 => name!("hash", "sha256"),
    FN_SHA512 => name!("hash", "sha512")
);

declare_fns!(default.ctx,
   FN_UUID => "uuid",
   FN_UUIDV5 => "uuidv5"
);
