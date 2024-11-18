declare_fns!(default.ctx,
    FN_FILE => name!("fs", "read"),
    FN_FILEMD5 => name!(["fs", "hash"] => "md5"),
    FN_FILESHA1 => name!(["fs", "hash"] => "sha1"),
    FN_FILESHA256 => name!(["fs", "hash"] => "sha256"),
    FN_FILESHA512 => name!(["fs", "hash"] => "sha512")
);
