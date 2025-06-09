// Copyright 2023-2024 The Open Sam Foundation (OSF)
// Developed by Caleb Mitchell Smith (PixelCoda)
// Licensed under GPLv3....see LICENSE file.


use laserfiche_rs::laserfiche;
use serde_json::json;

#[tokio::main]
async fn main() {
    let api = laserfiche::LFApiServer{address: format!("<HOST_ADDRESS>"), repository: format!("<repo>")};
    let auth = laserfiche::Auth::new(api.clone(), format!("<repo>"), format!("<password>")).await.unwrap();


    match auth{
        laserfiche::AuthOrError::Auth(ath) => {


            println!("{:?}", ath);

            let new = ath.refresh().await.unwrap();
            match new{
                laserfiche::AuthOrError::Auth(ath2) => {
                    println!("{:?}", ath2);
                },
                _ => {}
            }

        


            // let entry = laserfiche::Entry::list(api.clone(), ath.clone(), 1).await.unwrap();
            // match entry{
            //     laserfiche::EntriesOrError::Entries(entries) => {
            //         // println!("{:?}", ent);


            //         for ent in entries.value{
            //             let entry = laserfiche::Entry::get_metadata(api.clone(), ath.clone(), ent.id).await.unwrap();
            //             match entry{
            //                 laserfiche::MetadataResultOrError::Metadata(met) => {
            //                     println!("{:?}", met);
            //                     println!("{:?}", met.value[0].field_name);
            //                     println!("{:?}", met.value[0].values[0].value);
            //                     println!("{:?}", met.value.len());
            //                 },
            //                 laserfiche::MetadataResultOrError::LFAPIError(err) => println!("{:?}", err),
            //             }
            //         }
        


            //     },
            //     laserfiche::EntriesOrError::LFAPIError(err) => println!("{:?}", err),
            // }


            // Import file test
            let entry = laserfiche::Entry::import(api.clone(), ath.clone(), "incoming".to_string(), "test2.tiff".to_string(), 1).await.unwrap();
            match entry{
                laserfiche::ImportResultOrError::ImportResult(ent) => {
                    println!("{:?}", ent);
                },
                laserfiche::ImportResultOrError::LFAPIError(err) => println!("{:?}", err),
            }




            // // Export file test
            // let entry = laserfiche::Entry::export(api.clone(), ath.clone(), 3740, "export_test.png").await.unwrap();
            // match entry{
            //     laserfiche::BitsOrError::Bits(ent) => {
            //         println!("{:?}", ent);
            //     },
            //     laserfiche::BitsOrError::LFAPIError(err) => println!("{:?}", err),
            // }

        },
        laserfiche::AuthOrError::LFAPIError(err) => println!("{:?}", err),
    }
    
}


