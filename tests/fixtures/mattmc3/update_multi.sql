UPDATE products SET list_price = list_price + 100, modified_date = now() WHERE category = 'sale' AND active = TRUE
