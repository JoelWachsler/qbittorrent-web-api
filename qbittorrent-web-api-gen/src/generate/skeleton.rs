use quote::quote;

use super::util;

pub fn auth_ident() -> proc_macro2::Ident {
    util::to_ident("Authenticated")
}

pub fn generate_skeleton(ident: &syn::Ident) -> proc_macro2::TokenStream {
    let auth = auth_ident();

    quote! {
        impl super::#ident {
            /// Creates an authenticated client.
            /// base_url is the url to the qbittorrent instance, i.e. http://localhost:8080
            pub async fn login(
                base_url: &str,
                username: &str,
                password: &str,
            ) -> Result<#auth> {
                let client = reqwest::Client::new();

                let form = reqwest::multipart::Form::new()
                    .text("username", username.to_string())
                    .text("password", password.to_string());

                let auth_resp = client
                    .post(format!("{}/api/v2/auth/login", base_url))
                    .multipart(form)
                    .send()
                    .await?;

                let cookie_header = match auth_resp.headers().get("set-cookie") {
                    Some(header) => header.to_str().unwrap(),
                    None => {
                        return Err(Error::InvalidUsernameOrPassword);
                    }
                };

                fn parse_cookie(input: &str) -> Result<&str> {
                    match input.split(';').next() {
                        Some(res) => Ok(res),
                        _ => Err(Error::AuthCookieParseError),
                    }
                }

                let auth_cookie = parse_cookie(cookie_header)?;

                Ok(#auth {
                    client,
                    auth_cookie: auth_cookie.to_string(),
                    base_url: base_url.to_string(),
                })
            }
        }

        #[allow(clippy::enum_variant_names)]
        #[derive(Debug, thiserror::Error)]
        pub enum Error {
            #[error("failed to parse auth cookie")]
            AuthCookieParseError,
            #[error("invalid username or password (failed to parse auth cookie)")]
            InvalidUsernameOrPassword,
            #[error("request error: {0}")]
            HttpError(#[from] reqwest::Error),
        }

        type Result<T> = std::result::Result<T, Error>;

        #[derive(Debug)]
        pub struct #auth {
            auth_cookie: String,
            base_url: String,
            client: reqwest::Client,
        }

        impl #auth {
            fn authenticated_client(&self, url: &str) -> reqwest::RequestBuilder {
                let url = format!("{}{}", self.base_url, url);
                let cookie = self.auth_cookie.clone();

                self.client
                    .post(url)
                    .header("cookie", cookie)
            }

            pub async fn logout(self) -> Result<()> {
                self.authenticated_client("/api/v2/auth/logout")
                    .send()
                    .await?;

                Ok(())
            }
        }
    }
}
