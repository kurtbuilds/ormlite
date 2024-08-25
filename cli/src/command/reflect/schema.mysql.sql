    SELECT
        c.table_schema,
        c.table_name,
        c.column_name,
        false AS is_rev_fk,
        c.ordinal_position,

        c.is_nullable = 'YES' AS is_nullable,

        c.data_type AS type_name,

        false AS is_array,

        NULL AS domain_schema,
        NULL AS domain_name,
        c.column_key = 'PRI' AS is_primary_key,

        k.referenced_table_schema AS fk_table_schema,
        k.referenced_table_name AS fk_table_name,
        k.referenced_column_name AS fk_column_name,

        true AS is_updatable

    FROM information_schema.columns c
        LEFT JOIN information_schema.key_column_usage k
            ON c.table_schema = k.table_schema
            AND c.table_name = k.table_name
            AND c.column_name = k.column_name
    WHERE
        c.table_schema = $1

UNION

    SELECT
        k.referenced_table_schema AS table_schema,
        k.referenced_table_name AS table_name,
        k.referenced_column_name AS column_name,
        true AS is_rev_fk,
        k.position_in_unique_constraint AS ordinal_position,

        true AS is_nullable,

        '' AS type_name,

        true AS is_array,

        NULL AS domain_schema,
        NULL AS domain_name,
        false AS is_primary_key,

        k.table_schema AS fk_table_schema,
        k.table_name AS fk_table_name,
        k.column_name AS fk_column_name,

        false AS is_updatable

    FROM information_schema.key_column_usage k

    WHERE
        k.referenced_table_schema IS NOT NULL
        AND k.constraint_schema = $1

ORDER BY
    table_schema, table_name, is_rev_fk, ordinal_position;
