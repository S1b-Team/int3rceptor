fn main() {
    // License public key (for signature verification)
    // In production, this would be your actual Ed25519/RSA public key
    let license_public_key = "S1BGr0uP-Int3rceptor-v2.0-PublicKey-PLACEHOLDER";
    println!("cargo:rustc-env=LICENSE_PUBLIC_KEY={}", license_public_key);
}
