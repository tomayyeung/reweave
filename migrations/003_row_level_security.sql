CREATE OR REPLACE FUNCTION app_current_user_id()
RETURNS uuid
LANGUAGE sql
STABLE
AS $$
	SELECT NULLIF(current_setting('app.user_id', true), '')::uuid;
$$;

CREATE OR REPLACE FUNCTION app_current_clerk_user_id()
RETURNS text
LANGUAGE sql
STABLE
AS $$
	SELECT NULLIF(current_setting('app.clerk_user_id', true), '');
$$;

CREATE OR REPLACE FUNCTION app_current_role()
RETURNS text
LANGUAGE sql
STABLE
AS $$
	SELECT COALESCE(NULLIF(current_setting('app.role', true), ''), 'anonymous');
$$;

ALTER TABLE "users" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "users" FORCE ROW LEVEL SECURITY;
ALTER TABLE "puzzles" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "puzzles" FORCE ROW LEVEL SECURITY;
ALTER TABLE "puzzle_stats" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "puzzle_stats" FORCE ROW LEVEL SECURITY;
ALTER TABLE "puzzle_completion_events" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "puzzle_completion_events" FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS "users_select_public" ON "users";
CREATE POLICY "users_select_public" ON "users"
	FOR SELECT
	USING (true);

DROP POLICY IF EXISTS "users_insert_own_clerk_user" ON "users";
CREATE POLICY "users_insert_own_clerk_user" ON "users"
	FOR INSERT
	WITH CHECK (
		"clerk_user_id" IS NOT NULL
		AND "clerk_user_id" = app_current_clerk_user_id()
	);

DROP POLICY IF EXISTS "users_update_self" ON "users";
CREATE POLICY "users_update_self" ON "users"
	FOR UPDATE
	USING (
		"id" = app_current_user_id()
		OR (
			"clerk_user_id" IS NOT NULL
			AND "clerk_user_id" = app_current_clerk_user_id()
		)
	)
	WITH CHECK (
		"id" = app_current_user_id()
		OR (
			"clerk_user_id" IS NOT NULL
			AND "clerk_user_id" = app_current_clerk_user_id()
		)
	);

DROP POLICY IF EXISTS "puzzles_select_public" ON "puzzles";
CREATE POLICY "puzzles_select_public" ON "puzzles"
	FOR SELECT
	USING (true);

DROP POLICY IF EXISTS "puzzles_insert_creator" ON "puzzles";
CREATE POLICY "puzzles_insert_creator" ON "puzzles"
	FOR INSERT
	WITH CHECK ("created_by_user_id" = app_current_user_id());

DROP POLICY IF EXISTS "puzzles_update_creator_or_admin" ON "puzzles";
CREATE POLICY "puzzles_update_creator_or_admin" ON "puzzles"
	FOR UPDATE
	USING (
		"created_by_user_id" = app_current_user_id()
		OR app_current_role() = 'admin'
	)
	WITH CHECK (
		"created_by_user_id" = app_current_user_id()
		OR app_current_role() = 'admin'
	);

DROP POLICY IF EXISTS "puzzle_stats_select_public" ON "puzzle_stats";
CREATE POLICY "puzzle_stats_select_public" ON "puzzle_stats"
	FOR SELECT
	USING (true);

DROP POLICY IF EXISTS "puzzle_stats_insert_for_existing_puzzle" ON "puzzle_stats";
CREATE POLICY "puzzle_stats_insert_for_existing_puzzle" ON "puzzle_stats"
	FOR INSERT
	WITH CHECK (
		EXISTS (
			SELECT 1 FROM "puzzles" WHERE "puzzles"."id" = "puzzle_stats"."puzzle_id"
		)
	);

DROP POLICY IF EXISTS "puzzle_stats_update_backend" ON "puzzle_stats";
CREATE POLICY "puzzle_stats_update_backend" ON "puzzle_stats"
	FOR UPDATE
	USING (true)
	WITH CHECK (true);

DROP POLICY IF EXISTS "puzzle_completion_events_select_public" ON "puzzle_completion_events";
CREATE POLICY "puzzle_completion_events_select_public" ON "puzzle_completion_events"
	FOR SELECT
	USING (true);

DROP POLICY IF EXISTS "puzzle_completion_events_insert_self_or_anonymous" ON "puzzle_completion_events";
CREATE POLICY "puzzle_completion_events_insert_self_or_anonymous" ON "puzzle_completion_events"
	FOR INSERT
	WITH CHECK (
		"user_id" IS NULL
		OR "user_id" = app_current_user_id()
	);
