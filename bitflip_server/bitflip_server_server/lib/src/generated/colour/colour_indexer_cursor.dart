/* AUTOMATICALLY GENERATED CODE DO NOT MODIFY */
/*   To generate run: "serverpod generate"    */

// ignore_for_file: implementation_imports
// ignore_for_file: library_private_types_in_public_api
// ignore_for_file: non_constant_identifier_names
// ignore_for_file: public_member_api_docs
// ignore_for_file: type_literal_in_constant_pattern
// ignore_for_file: use_super_parameters
// ignore_for_file: invalid_use_of_internal_member

// ignore_for_file: no_leading_underscores_for_library_prefixes
import 'package:serverpod/serverpod.dart' as _is;

abstract class ColourIndexerCursor
    implements _is.TableRow<int?>, _is.ProtocolSerialization {
  ColourIndexerCursor._({
    this.id,
    required this.cluster,
    required this.programAddress,
    required this.startSignature,
    required this.completedHeadSignature,
    this.catchUpHeadSignature,
    this.beforeSignature,
    this.leaseToken,
    this.leasedUntil,
    this.lastSuccessAt,
    required this.updatedAt,
  });

  factory ColourIndexerCursor({
    int? id,
    required String cluster,
    required String programAddress,
    required String startSignature,
    required String completedHeadSignature,
    String? catchUpHeadSignature,
    String? beforeSignature,
    String? leaseToken,
    DateTime? leasedUntil,
    DateTime? lastSuccessAt,
    required DateTime updatedAt,
  }) = _ColourIndexerCursorImpl;

  factory ColourIndexerCursor.fromJson(Map<String, dynamic> jsonSerialization) {
    return ColourIndexerCursor(
      id: jsonSerialization['id'] as int?,
      cluster: jsonSerialization['cluster'] as String,
      programAddress: jsonSerialization['programAddress'] as String,
      startSignature: jsonSerialization['startSignature'] as String,
      completedHeadSignature:
          jsonSerialization['completedHeadSignature'] as String,
      catchUpHeadSignature:
          jsonSerialization['catchUpHeadSignature'] as String?,
      beforeSignature: jsonSerialization['beforeSignature'] as String?,
      leaseToken: jsonSerialization['leaseToken'] as String?,
      leasedUntil: jsonSerialization['leasedUntil'] == null
          ? null
          : _is.DateTimeJsonExtension.fromJson(
              jsonSerialization['leasedUntil'],
            ),
      lastSuccessAt: jsonSerialization['lastSuccessAt'] == null
          ? null
          : _is.DateTimeJsonExtension.fromJson(
              jsonSerialization['lastSuccessAt'],
            ),
      updatedAt: _is.DateTimeJsonExtension.fromJson(
        jsonSerialization['updatedAt'],
      ),
    );
  }

  static final t = ColourIndexerCursorTable();

  static const db = ColourIndexerCursorRepository._();

  @override
  int? id;

  String cluster;

  String programAddress;

  String startSignature;

  String completedHeadSignature;

  String? catchUpHeadSignature;

  String? beforeSignature;

  String? leaseToken;

  DateTime? leasedUntil;

  DateTime? lastSuccessAt;

  DateTime updatedAt;

  @override
  _is.Table<int?> get table => t;

  /// Returns a shallow copy of this [ColourIndexerCursor]
  /// with some or all fields replaced by the given arguments.
  @_is.useResult
  ColourIndexerCursor copyWith({
    int? id,
    String? cluster,
    String? programAddress,
    String? startSignature,
    String? completedHeadSignature,
    String? catchUpHeadSignature,
    String? beforeSignature,
    String? leaseToken,
    DateTime? leasedUntil,
    DateTime? lastSuccessAt,
    DateTime? updatedAt,
  });
  @override
  Map<String, dynamic> toJson() {
    return {
      '__className__': 'ColourIndexerCursor',
      if (id != null) 'id': id,
      'cluster': cluster,
      'programAddress': programAddress,
      'startSignature': startSignature,
      'completedHeadSignature': completedHeadSignature,
      if (catchUpHeadSignature != null)
        'catchUpHeadSignature': catchUpHeadSignature,
      if (beforeSignature != null) 'beforeSignature': beforeSignature,
      if (leaseToken != null) 'leaseToken': leaseToken,
      if (leasedUntil != null) 'leasedUntil': leasedUntil?.toJson(),
      if (lastSuccessAt != null) 'lastSuccessAt': lastSuccessAt?.toJson(),
      'updatedAt': updatedAt.toJson(),
    };
  }

  @override
  Map<String, dynamic> toJsonForProtocol() {
    return {};
  }

  static ColourIndexerCursorInclude include() {
    return ColourIndexerCursorInclude._();
  }

  static ColourIndexerCursorIncludeList includeList({
    _is.WhereExpressionBuilder<ColourIndexerCursorTable>? where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<ColourIndexerCursorTable>? orderBy,
    _is.OrderByListBuilder<ColourIndexerCursorTable>? orderByList,
    ColourIndexerCursorInclude? include,
  }) {
    return ColourIndexerCursorIncludeList._(
      where: where,
      limit: limit,
      offset: offset,
      orderBy: orderBy?.call(ColourIndexerCursor.t),
      orderByList: orderByList?.call(ColourIndexerCursor.t),
      include: include,
    );
  }

  @override
  String toString() {
    return _is.SerializationManager.encode(this);
  }
}

class _Undefined {}

class _ColourIndexerCursorImpl extends ColourIndexerCursor {
  _ColourIndexerCursorImpl({
    int? id,
    required String cluster,
    required String programAddress,
    required String startSignature,
    required String completedHeadSignature,
    String? catchUpHeadSignature,
    String? beforeSignature,
    String? leaseToken,
    DateTime? leasedUntil,
    DateTime? lastSuccessAt,
    required DateTime updatedAt,
  }) : super._(
         id: id,
         cluster: cluster,
         programAddress: programAddress,
         startSignature: startSignature,
         completedHeadSignature: completedHeadSignature,
         catchUpHeadSignature: catchUpHeadSignature,
         beforeSignature: beforeSignature,
         leaseToken: leaseToken,
         leasedUntil: leasedUntil,
         lastSuccessAt: lastSuccessAt,
         updatedAt: updatedAt,
       );

  /// Returns a shallow copy of this [ColourIndexerCursor]
  /// with some or all fields replaced by the given arguments.
  @_is.useResult
  @override
  ColourIndexerCursor copyWith({
    Object? id = _Undefined,
    String? cluster,
    String? programAddress,
    String? startSignature,
    String? completedHeadSignature,
    Object? catchUpHeadSignature = _Undefined,
    Object? beforeSignature = _Undefined,
    Object? leaseToken = _Undefined,
    Object? leasedUntil = _Undefined,
    Object? lastSuccessAt = _Undefined,
    DateTime? updatedAt,
  }) {
    return ColourIndexerCursor(
      id: id is int? ? id : this.id,
      cluster: cluster ?? this.cluster,
      programAddress: programAddress ?? this.programAddress,
      startSignature: startSignature ?? this.startSignature,
      completedHeadSignature:
          completedHeadSignature ?? this.completedHeadSignature,
      catchUpHeadSignature: catchUpHeadSignature is String?
          ? catchUpHeadSignature
          : this.catchUpHeadSignature,
      beforeSignature: beforeSignature is String?
          ? beforeSignature
          : this.beforeSignature,
      leaseToken: leaseToken is String? ? leaseToken : this.leaseToken,
      leasedUntil: leasedUntil is DateTime? ? leasedUntil : this.leasedUntil,
      lastSuccessAt: lastSuccessAt is DateTime?
          ? lastSuccessAt
          : this.lastSuccessAt,
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }
}

class ColourIndexerCursorUpdateTable
    extends _is.UpdateTable<ColourIndexerCursorTable> {
  ColourIndexerCursorUpdateTable(super.table);

  _is.ColumnValue<String, String> cluster(String value) => _is.ColumnValue(
    table.cluster,
    value,
  );

  _is.ColumnValue<String, String> programAddress(String value) =>
      _is.ColumnValue(
        table.programAddress,
        value,
      );

  _is.ColumnValue<String, String> startSignature(String value) =>
      _is.ColumnValue(
        table.startSignature,
        value,
      );

  _is.ColumnValue<String, String> completedHeadSignature(String value) =>
      _is.ColumnValue(
        table.completedHeadSignature,
        value,
      );

  _is.ColumnValue<String, String> catchUpHeadSignature(String? value) =>
      _is.ColumnValue(
        table.catchUpHeadSignature,
        value,
      );

  _is.ColumnValue<String, String> beforeSignature(String? value) =>
      _is.ColumnValue(
        table.beforeSignature,
        value,
      );

  _is.ColumnValue<String, String> leaseToken(String? value) => _is.ColumnValue(
    table.leaseToken,
    value,
  );

  _is.ColumnValue<DateTime, DateTime> leasedUntil(DateTime? value) =>
      _is.ColumnValue(
        table.leasedUntil,
        value,
      );

  _is.ColumnValue<DateTime, DateTime> lastSuccessAt(DateTime? value) =>
      _is.ColumnValue(
        table.lastSuccessAt,
        value,
      );

  _is.ColumnValue<DateTime, DateTime> updatedAt(DateTime value) =>
      _is.ColumnValue(
        table.updatedAt,
        value,
      );
}

class ColourIndexerCursorTable extends _is.Table<int?> {
  ColourIndexerCursorTable({super.tableRelation})
    : super(tableName: 'bitflip_colour_indexer_cursor') {
    updateTable = ColourIndexerCursorUpdateTable(this);
    cluster = _is.ColumnString(
      'cluster',
      this,
    );
    programAddress = _is.ColumnString(
      'programAddress',
      this,
    );
    startSignature = _is.ColumnString(
      'startSignature',
      this,
    );
    completedHeadSignature = _is.ColumnString(
      'completedHeadSignature',
      this,
    );
    catchUpHeadSignature = _is.ColumnString(
      'catchUpHeadSignature',
      this,
    );
    beforeSignature = _is.ColumnString(
      'beforeSignature',
      this,
    );
    leaseToken = _is.ColumnString(
      'leaseToken',
      this,
    );
    leasedUntil = _is.ColumnDateTime(
      'leasedUntil',
      this,
    );
    lastSuccessAt = _is.ColumnDateTime(
      'lastSuccessAt',
      this,
    );
    updatedAt = _is.ColumnDateTime(
      'updatedAt',
      this,
    );
  }

  late final ColourIndexerCursorUpdateTable updateTable;

  late final _is.ColumnString cluster;

  late final _is.ColumnString programAddress;

  late final _is.ColumnString startSignature;

  late final _is.ColumnString completedHeadSignature;

  late final _is.ColumnString catchUpHeadSignature;

  late final _is.ColumnString beforeSignature;

  late final _is.ColumnString leaseToken;

  late final _is.ColumnDateTime leasedUntil;

  late final _is.ColumnDateTime lastSuccessAt;

  late final _is.ColumnDateTime updatedAt;

  @override
  List<_is.Column> get columns => [
    id,
    cluster,
    programAddress,
    startSignature,
    completedHeadSignature,
    catchUpHeadSignature,
    beforeSignature,
    leaseToken,
    leasedUntil,
    lastSuccessAt,
    updatedAt,
  ];
}

class ColourIndexerCursorInclude extends _is.IncludeObject {
  ColourIndexerCursorInclude._();

  @override
  Map<String, _is.Include?> get includes => {};

  @override
  _is.Table<int?> get table => ColourIndexerCursor.t;
}

class ColourIndexerCursorIncludeList extends _is.IncludeList {
  ColourIndexerCursorIncludeList._({
    _is.WhereExpressionBuilder<ColourIndexerCursorTable>? where,
    super.limit,
    super.offset,
    super.orderBy,
    super.orderByList,
    super.include,
  }) {
    super.where = where?.call(ColourIndexerCursor.t);
  }

  @override
  Map<String, _is.Include?> get includes => include?.includes ?? {};

  @override
  _is.Table<int?> get table => ColourIndexerCursor.t;
}

class ColourIndexerCursorRepository {
  const ColourIndexerCursorRepository._();

  /// Returns a list of [ColourIndexerCursor]s matching the given query parameters.
  ///
  /// Use [where] to specify which items to include in the return value.
  /// If none is specified, all items will be returned.
  ///
  /// To specify the order of the items use [orderBy] or [orderByList]
  /// when sorting by multiple columns.
  ///
  /// The maximum number of items can be set by [limit]. If no limit is set,
  /// all items matching the query will be returned.
  ///
  /// [offset] defines how many items to skip, after which [limit] (or all)
  /// items are read from the database.
  ///
  /// ```dart
  /// var persons = await Persons.db.find(
  ///   session,
  ///   where: (t) => t.lastName.equals('Jones'),
  ///   orderBy: (t) => t.firstName,
  ///   limit: 100,
  /// );
  /// ```
  Future<List<ColourIndexerCursor>> find(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<ColourIndexerCursorTable>? where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<ColourIndexerCursorTable>? orderBy,
    _is.OrderByListBuilder<ColourIndexerCursorTable>? orderByList,
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.find<ColourIndexerCursor>(
      where: where?.call(ColourIndexerCursor.t),
      orderBy: orderBy?.call(ColourIndexerCursor.t),
      orderByList: orderByList?.call(ColourIndexerCursor.t),
      limit: limit,
      offset: offset,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Returns the first matching [ColourIndexerCursor] matching the given query parameters.
  ///
  /// Use [where] to specify which items to include in the return value.
  /// If none is specified, all items will be returned.
  ///
  /// To specify the order use [orderBy] or [orderByList]
  /// when sorting by multiple columns.
  ///
  /// [offset] defines how many items to skip, after which the next one will be picked.
  ///
  /// ```dart
  /// var youngestPerson = await Persons.db.findFirstRow(
  ///   session,
  ///   where: (t) => t.lastName.equals('Jones'),
  ///   orderBy: (t) => t.age,
  /// );
  /// ```
  Future<ColourIndexerCursor?> findFirstRow(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<ColourIndexerCursorTable>? where,
    int? offset,
    _is.OrderByBuilder<ColourIndexerCursorTable>? orderBy,
    _is.OrderByListBuilder<ColourIndexerCursorTable>? orderByList,
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.findFirstRow<ColourIndexerCursor>(
      where: where?.call(ColourIndexerCursor.t),
      orderBy: orderBy?.call(ColourIndexerCursor.t),
      orderByList: orderByList?.call(ColourIndexerCursor.t),
      offset: offset,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Finds a single [ColourIndexerCursor] by its [id] or null if no such row exists.
  Future<ColourIndexerCursor?> findById(
    _is.DatabaseSession session,
    int id, {
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.findById<ColourIndexerCursor>(
      id,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Inserts all [ColourIndexerCursor]s in the list and returns the inserted rows.
  ///
  /// The returned [ColourIndexerCursor]s will have their `id` fields set.
  ///
  /// This is an atomic operation, meaning that if one of the rows fails to
  /// insert, none of the rows will be inserted.
  ///
  /// If [ignoreConflicts] is set to `true`, rows that conflict with existing
  /// rows are silently skipped, and only the successfully inserted rows are
  /// returned.
  ///
  /// If [noReturn] is set to `true`, the inserted rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ColourIndexerCursor>> insert(
    _is.DatabaseSession session,
    List<ColourIndexerCursor> rows, {
    _is.Transaction? transaction,
    bool ignoreConflicts = false,
    bool noReturn = false,
  }) async {
    return session.db.insert<ColourIndexerCursor>(
      rows,
      transaction: transaction,
      ignoreConflicts: ignoreConflicts,
      noReturn: noReturn,
    );
  }

  /// Inserts a single [ColourIndexerCursor] and returns the inserted row.
  ///
  /// The returned [ColourIndexerCursor] will have its `id` field set.
  Future<ColourIndexerCursor> insertRow(
    _is.DatabaseSession session,
    ColourIndexerCursor row, {
    _is.Transaction? transaction,
  }) async {
    return session.db.insertRow<ColourIndexerCursor>(
      row,
      transaction: transaction,
    );
  }

  /// Upserts all [ColourIndexerCursor]s in the list and returns the resulting rows.
  ///
  /// If a row conflicts on the given [conflictColumns], the existing row is
  /// updated with the new values. Otherwise, a new row is inserted.
  ///
  /// If [updateColumns] is provided, only those columns will be updated on
  /// conflict. If null, all non-conflict, non-id columns are updated.
  ///
  /// If [updateWhere] is provided, the update only applies to rows matching the
  /// given expression. Conflicting rows that don't match are skipped and not
  /// returned, so the resulting list may be shorter than [rows].
  ///
  /// The returned [ColourIndexerCursor]s will have their `id` fields set.
  ///
  /// This is an atomic operation, meaning that if one of the rows fails,
  /// none of the rows will be affected.
  ///
  /// If [noReturn] is set to `true`, the resulting rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ColourIndexerCursor>> upsert(
    _is.DatabaseSession session,
    List<ColourIndexerCursor> rows, {
    required _is.ColumnSelections<ColourIndexerCursorTable> conflictColumns,
    _is.ColumnSelections<ColourIndexerCursorTable>? updateColumns,
    _is.WhereExpressionBuilder<ColourIndexerCursorTable>? updateWhere,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.upsert<ColourIndexerCursor>(
      rows,
      conflictColumns: conflictColumns(ColourIndexerCursor.t),
      updateColumns: updateColumns?.call(ColourIndexerCursor.t),
      updateWhere: updateWhere?.call(ColourIndexerCursor.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Upserts a single [ColourIndexerCursor] and returns the resulting row.
  ///
  /// If the row conflicts on the given [conflictColumns], the existing row is
  /// updated. Otherwise, a new row is inserted.
  ///
  /// If [updateColumns] is provided, only those columns will be updated on
  /// conflict. If null, all non-conflict, non-id columns are updated.
  ///
  /// If [updateWhere] is provided, the update only applies when the existing
  /// row matches the expression. Returns `null` if no row was affected — for
  /// example when [updateWhere] does not match the conflicting row.
  ///
  /// The returned [ColourIndexerCursor] will have its `id` field set.
  Future<ColourIndexerCursor?> upsertRow(
    _is.DatabaseSession session,
    ColourIndexerCursor row, {
    required _is.ColumnSelections<ColourIndexerCursorTable> conflictColumns,
    _is.ColumnSelections<ColourIndexerCursorTable>? updateColumns,
    _is.WhereExpressionBuilder<ColourIndexerCursorTable>? updateWhere,
    _is.Transaction? transaction,
  }) async {
    return session.db.upsertRow<ColourIndexerCursor>(
      row,
      conflictColumns: conflictColumns(ColourIndexerCursor.t),
      updateColumns: updateColumns?.call(ColourIndexerCursor.t),
      updateWhere: updateWhere?.call(ColourIndexerCursor.t),
      transaction: transaction,
    );
  }

  /// Updates all [ColourIndexerCursor]s in the list and returns the updated rows. If
  /// [columns] is provided, only those columns will be updated. Defaults to
  /// all columns.
  /// This is an atomic operation, meaning that if one of the rows fails to
  /// update, none of the rows will be updated.
  ///
  /// If [noReturn] is set to `true`, the updated rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ColourIndexerCursor>> update(
    _is.DatabaseSession session,
    List<ColourIndexerCursor> rows, {
    _is.ColumnSelections<ColourIndexerCursorTable>? columns,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.update<ColourIndexerCursor>(
      rows,
      columns: columns?.call(ColourIndexerCursor.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Updates a single [ColourIndexerCursor]. The row needs to have its id set.
  /// Optionally, a list of [columns] can be provided to only update those
  /// columns. Defaults to all columns.
  Future<ColourIndexerCursor> updateRow(
    _is.DatabaseSession session,
    ColourIndexerCursor row, {
    _is.ColumnSelections<ColourIndexerCursorTable>? columns,
    _is.Transaction? transaction,
  }) async {
    return session.db.updateRow<ColourIndexerCursor>(
      row,
      columns: columns?.call(ColourIndexerCursor.t),
      transaction: transaction,
    );
  }

  /// Updates a single [ColourIndexerCursor] by its [id] with the specified [columnValues].
  /// Returns the updated row or null if no row with the given id exists.
  Future<ColourIndexerCursor?> updateById(
    _is.DatabaseSession session,
    int id, {
    required _is.ColumnValueListBuilder<ColourIndexerCursorUpdateTable>
    columnValues,
    _is.Transaction? transaction,
  }) async {
    return session.db.updateById<ColourIndexerCursor>(
      id,
      columnValues: columnValues(ColourIndexerCursor.t.updateTable),
      transaction: transaction,
    );
  }

  /// Updates all [ColourIndexerCursor]s matching the [where] expression with the specified [columnValues].
  /// Returns the list of updated rows.
  ///
  /// If [noReturn] is set to `true`, the updated rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ColourIndexerCursor>> updateWhere(
    _is.DatabaseSession session, {
    required _is.ColumnValueListBuilder<ColourIndexerCursorUpdateTable>
    columnValues,
    required _is.WhereExpressionBuilder<ColourIndexerCursorTable> where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<ColourIndexerCursorTable>? orderBy,
    _is.OrderByListBuilder<ColourIndexerCursorTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.updateWhere<ColourIndexerCursor>(
      columnValues: columnValues(ColourIndexerCursor.t.updateTable),
      where: where(ColourIndexerCursor.t),
      limit: limit,
      offset: offset,
      orderBy: orderBy?.call(ColourIndexerCursor.t),
      orderByList: orderByList?.call(ColourIndexerCursor.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Deletes all [ColourIndexerCursor]s in the list and returns the deleted rows.
  ///
  /// To specify the order of the returned rows use [orderBy] or [orderByList]
  /// when sorting by multiple columns.
  ///
  /// This is an atomic operation, meaning that if one of the rows fail to
  /// be deleted, none of the rows will be deleted.
  ///
  /// If [noReturn] is set to `true`, the deleted rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ColourIndexerCursor>> delete(
    _is.DatabaseSession session,
    List<ColourIndexerCursor> rows, {
    _is.OrderByBuilder<ColourIndexerCursorTable>? orderBy,
    _is.OrderByListBuilder<ColourIndexerCursorTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.delete<ColourIndexerCursor>(
      rows,
      orderBy: orderBy?.call(ColourIndexerCursor.t),
      orderByList: orderByList?.call(ColourIndexerCursor.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Deletes a single [ColourIndexerCursor].
  Future<ColourIndexerCursor> deleteRow(
    _is.DatabaseSession session,
    ColourIndexerCursor row, {
    _is.Transaction? transaction,
  }) async {
    return session.db.deleteRow<ColourIndexerCursor>(
      row,
      transaction: transaction,
    );
  }

  /// Deletes all rows matching the [where] expression.
  ///
  /// To specify the order of the returned rows use [orderBy] or [orderByList]
  /// when sorting by multiple columns.
  ///
  /// If [noReturn] is set to `true`, the deleted rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<ColourIndexerCursor>> deleteWhere(
    _is.DatabaseSession session, {
    required _is.WhereExpressionBuilder<ColourIndexerCursorTable> where,
    _is.OrderByBuilder<ColourIndexerCursorTable>? orderBy,
    _is.OrderByListBuilder<ColourIndexerCursorTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.deleteWhere<ColourIndexerCursor>(
      where: where(ColourIndexerCursor.t),
      orderBy: orderBy?.call(ColourIndexerCursor.t),
      orderByList: orderByList?.call(ColourIndexerCursor.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Counts the number of rows matching the [where] expression. If omitted,
  /// will return the count of all rows in the table.
  Future<int> count(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<ColourIndexerCursorTable>? where,
    int? limit,
    _is.Transaction? transaction,
  }) async {
    return session.db.count<ColourIndexerCursor>(
      where: where?.call(ColourIndexerCursor.t),
      limit: limit,
      transaction: transaction,
    );
  }

  /// Acquires row-level locks on [ColourIndexerCursor] rows matching the [where] expression.
  Future<void> lockRows(
    _is.DatabaseSession session, {
    required _is.WhereExpressionBuilder<ColourIndexerCursorTable> where,
    required _is.LockMode lockMode,
    required _is.Transaction transaction,
    _is.LockBehavior lockBehavior = _is.LockBehavior.wait,
  }) async {
    return session.db.lockRows<ColourIndexerCursor>(
      where: where(ColourIndexerCursor.t),
      lockMode: lockMode,
      lockBehavior: lockBehavior,
      transaction: transaction,
    );
  }
}
