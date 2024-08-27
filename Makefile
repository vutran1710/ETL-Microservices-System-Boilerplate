APP_NAME := $(name)
TABLES := $(tables)
TABLES_FORMATTED := $(shell echo $(TABLES) | sed 's/[^,]*/"&"/g' | sed 's/,/, /g')

create-etl:
	@echo "Creating ETL for app: $(APP_NAME)"
	@echo "Using tables: $(TABLES)"
	# Duplicate the template crate and replace all instances of __template__ with the app name
	@cp -r ./crates/__template__ ./crates/$(APP_NAME)

	# Replace all instances of __template__ with the app name
	@find ./crates/$(APP_NAME) -type f -exec sed -i '' 's/__template__/$(APP_NAME)/g' {} +
	echo "Appending database configuration to Cargo.toml with tables: $(TABLES_FORMATTED)"; \
	echo 'database = { workspace = true, features = [$(TABLES_FORMATTED)] }' >> ./crates/$(APP_NAME)/Cargo.toml

	# Finish
	@echo "ETL for app $(APP_NAME) created successfully with tables: $(TABLES)"

	# Manual steps
	@echo "Please remember to add your new new lib to the workspace Cargo.toml"
	@echo "Please remember to add your new new lib to the etl-app/Cargo.toml"
