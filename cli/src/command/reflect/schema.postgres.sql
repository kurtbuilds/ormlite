    SELECT
        nc.nspname AS table_schema,
        c.relname AS table_name,
        a.attname AS column_name,
        false AS is_rev_fk,
        a.attnum AS ordinal_position,

        NOT(CASE WHEN t.typtype = 'd' THEN t.typnotnull ELSE a.attnotnull END) AS is_nullable,

        REGEXP_REPLACE(CASE WHEN t.typtype = 'd' THEN bt.typname ELSE t.typname END, '^_', '') type_name,

        CASE
            WHEN t.typtype = 'd' THEN
                bt.typelem <> 0 AND bt.typlen = '-1'::integer
            ELSE
                t.typelem <> 0 AND t.typlen = '-1'::integer
        END AS is_array,

        CASE WHEN t.typtype = 'd' THEN nt.nspname ELSE NULL END AS domain_schema,
        CASE WHEN t.typtype = 'd' THEN t.typname ELSE NULL END AS domain_name,
        COALESCE(i.indisprimary, false) AS is_primary_key,

        fnc.nspname AS fk_table_schema,
        fc.relname AS fk_table_name,
        fa.attname AS fk_column_name,

        (c.relkind IN ('r', 'p')) OR (c.relkind IN ('v', 'f')) AND pg_column_is_updatable(c.oid, a.attnum, false) AS is_updatable
    FROM pg_attribute a
        JOIN (pg_class c JOIN pg_namespace nc ON c.relnamespace = nc.oid) ON a.attrelid = c.oid
        JOIN (pg_type t JOIN pg_namespace nt ON t.typnamespace = nt.oid) ON a.atttypid = t.oid
        LEFT JOIN (pg_type bt JOIN pg_namespace nbt ON bt.typnamespace = nbt.oid) ON t.typbasetype = bt.oid AND t.typtype = 'd'
        LEFT JOIN pg_attrdef ad ON a.attrelid = ad.adrelid AND a.attnum = ad.adnum
        LEFT JOIN pg_index i ON a.attrelid = i.indrelid AND i.indisprimary AND a.attnum = ANY(i.indkey)
        LEFT JOIN pg_constraint n ON n.contype = 'f' AND n.conrelid = a.attrelid AND a.attnum = ANY(n.conkey)
        LEFT JOIN pg_attribute fa ON n.confrelid = fa.attrelid AND fa.attnum = n.confkey[ARRAY_POSITION(n.conkey, a.attnum)] AND NOT fa.attisdropped
        LEFT JOIN (pg_class fc JOIN pg_namespace fnc ON fc.relnamespace = fnc.oid) ON fa.attrelid = fc.oid
    WHERE
        NOT pg_is_other_temp_schema(nc.oid)
        AND a.attnum > 0
        AND NOT a.attisdropped
        AND (c.relkind IN ('r', 'v', 'f', 'p'))
        AND (pg_has_role(c.relowner, 'USAGE') OR has_column_privilege(c.oid, a.attnum, 'SELECT, INSERT, UPDATE, REFERENCES'))
    AND nc.nspname = $1

UNION

    SELECT
        nc.nspname AS table_schema,
        c.relname AS table_name,
        a.attname AS column_name,
        true AS is_rev_fk,
        a.attnum AS ordinal_position,

        true AS is_nullable,

        '' AS type_name,

        true AS is_array,

        NULL AS domain_schema,
        NULL AS domain_name,
        false AS is_primary_key,

        rfnc.nspname AS fk_table_schema,
        rfc.relname AS fk_table_name,
        rfa.attname AS fk_column_name,

        false AS is_updatable
    FROM pg_attribute a
        JOIN (pg_class c JOIN pg_namespace nc ON c.relnamespace = nc.oid) ON a.attrelid = c.oid
        INNER JOIN pg_constraint rn ON rn.contype = 'f' AND rn.confrelid = a.attrelid AND a.attnum = ANY(rn.confkey)
        INNER JOIN pg_attribute rfa ON rn.conrelid = rfa.attrelid AND rfa.attnum = rn.conkey[ARRAY_POSITION(rn.confkey, a.attnum)] AND NOT rfa.attisdropped
        INNER JOIN (pg_class rfc JOIN pg_namespace rfnc ON rfc.relnamespace = rfnc.oid) ON rfa.attrelid = rfc.oid
    WHERE
        NOT pg_is_other_temp_schema(nc.oid)
        AND a.attnum > 0
        AND NOT a.attisdropped
        AND (pg_has_role(c.relowner, 'USAGE') OR has_column_privilege(c.oid, a.attnum, 'SELECT, INSERT, UPDATE, REFERENCES'))
        AND nc.nspname = $1

ORDER BY
    table_schema, table_name, is_rev_fk, ordinal_position;
