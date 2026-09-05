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

abstract class MintChallenge
    implements _is.TableRow<_is.UuidValue?>, _is.ProtocolSerialization {
  MintChallenge._({
    this.id,
    required this.walletAddress,
    required this.gameIndex,
    required this.sectionIndex,
    required this.nonce,
    required this.message,
    DateTime? createdAt,
    required this.expiresAt,
    this.usedAt,
  }) : createdAt = createdAt ?? DateTime.now();

  factory MintChallenge({
    _is.UuidValue? id,
    required String walletAddress,
    required int gameIndex,
    required int sectionIndex,
    required String nonce,
    required String message,
    DateTime? createdAt,
    required DateTime expiresAt,
    DateTime? usedAt,
  }) = _MintChallengeImpl;

  factory MintChallenge.fromJson(Map<String, dynamic> jsonSerialization) {
    return MintChallenge(
      id: jsonSerialization['id'] == null
          ? null
          : _is.UuidValueJsonExtension.fromJson(jsonSerialization['id']),
      walletAddress: jsonSerialization['walletAddress'] as String,
      gameIndex: jsonSerialization['gameIndex'] as int,
      sectionIndex: jsonSerialization['sectionIndex'] as int,
      nonce: jsonSerialization['nonce'] as String,
      message: jsonSerialization['message'] as String,
      createdAt: jsonSerialization['createdAt'] == null
          ? null
          : _is.DateTimeJsonExtension.fromJson(jsonSerialization['createdAt']),
      expiresAt: _is.DateTimeJsonExtension.fromJson(
        jsonSerialization['expiresAt'],
      ),
      usedAt: jsonSerialization['usedAt'] == null
          ? null
          : _is.DateTimeJsonExtension.fromJson(jsonSerialization['usedAt']),
    );
  }

  static final t = MintChallengeTable();

  static const db = MintChallengeRepository._();

  @override
  _is.UuidValue? id;

  String walletAddress;

  int gameIndex;

  int sectionIndex;

  String nonce;

  String message;

  DateTime createdAt;

  DateTime expiresAt;

  DateTime? usedAt;

  @override
  _is.Table<_is.UuidValue?> get table => t;

  /// Returns a shallow copy of this [MintChallenge]
  /// with some or all fields replaced by the given arguments.
  @_is.useResult
  MintChallenge copyWith({
    _is.UuidValue? id,
    String? walletAddress,
    int? gameIndex,
    int? sectionIndex,
    String? nonce,
    String? message,
    DateTime? createdAt,
    DateTime? expiresAt,
    DateTime? usedAt,
  });
  @override
  Map<String, dynamic> toJson() {
    return {
      '__className__': 'MintChallenge',
      if (id != null) 'id': id?.toJson(),
      'walletAddress': walletAddress,
      'gameIndex': gameIndex,
      'sectionIndex': sectionIndex,
      'nonce': nonce,
      'message': message,
      'createdAt': createdAt.toJson(),
      'expiresAt': expiresAt.toJson(),
      if (usedAt != null) 'usedAt': usedAt?.toJson(),
    };
  }

  @override
  Map<String, dynamic> toJsonForProtocol() {
    return {};
  }

  static MintChallengeInclude include() {
    return MintChallengeInclude._();
  }

  static MintChallengeIncludeList includeList({
    _is.WhereExpressionBuilder<MintChallengeTable>? where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<MintChallengeTable>? orderBy,
    _is.OrderByListBuilder<MintChallengeTable>? orderByList,
    MintChallengeInclude? include,
  }) {
    return MintChallengeIncludeList._(
      where: where,
      limit: limit,
      offset: offset,
      orderBy: orderBy?.call(MintChallenge.t),
      orderByList: orderByList?.call(MintChallenge.t),
      include: include,
    );
  }

  @override
  String toString() {
    return _is.SerializationManager.encode(this);
  }
}

class _Undefined {}

class _MintChallengeImpl extends MintChallenge {
  _MintChallengeImpl({
    _is.UuidValue? id,
    required String walletAddress,
    required int gameIndex,
    required int sectionIndex,
    required String nonce,
    required String message,
    DateTime? createdAt,
    required DateTime expiresAt,
    DateTime? usedAt,
  }) : super._(
         id: id,
         walletAddress: walletAddress,
         gameIndex: gameIndex,
         sectionIndex: sectionIndex,
         nonce: nonce,
         message: message,
         createdAt: createdAt,
         expiresAt: expiresAt,
         usedAt: usedAt,
       );

  /// Returns a shallow copy of this [MintChallenge]
  /// with some or all fields replaced by the given arguments.
  @_is.useResult
  @override
  MintChallenge copyWith({
    Object? id = _Undefined,
    String? walletAddress,
    int? gameIndex,
    int? sectionIndex,
    String? nonce,
    String? message,
    DateTime? createdAt,
    DateTime? expiresAt,
    Object? usedAt = _Undefined,
  }) {
    return MintChallenge(
      id: id is _is.UuidValue? ? id : this.id,
      walletAddress: walletAddress ?? this.walletAddress,
      gameIndex: gameIndex ?? this.gameIndex,
      sectionIndex: sectionIndex ?? this.sectionIndex,
      nonce: nonce ?? this.nonce,
      message: message ?? this.message,
      createdAt: createdAt ?? this.createdAt,
      expiresAt: expiresAt ?? this.expiresAt,
      usedAt: usedAt is DateTime? ? usedAt : this.usedAt,
    );
  }
}

class MintChallengeUpdateTable extends _is.UpdateTable<MintChallengeTable> {
  MintChallengeUpdateTable(super.table);

  _is.ColumnValue<String, String> walletAddress(String value) =>
      _is.ColumnValue(
        table.walletAddress,
        value,
      );

  _is.ColumnValue<int, int> gameIndex(int value) => _is.ColumnValue(
    table.gameIndex,
    value,
  );

  _is.ColumnValue<int, int> sectionIndex(int value) => _is.ColumnValue(
    table.sectionIndex,
    value,
  );

  _is.ColumnValue<String, String> nonce(String value) => _is.ColumnValue(
    table.nonce,
    value,
  );

  _is.ColumnValue<String, String> message(String value) => _is.ColumnValue(
    table.message,
    value,
  );

  _is.ColumnValue<DateTime, DateTime> createdAt(DateTime value) =>
      _is.ColumnValue(
        table.createdAt,
        value,
      );

  _is.ColumnValue<DateTime, DateTime> expiresAt(DateTime value) =>
      _is.ColumnValue(
        table.expiresAt,
        value,
      );

  _is.ColumnValue<DateTime, DateTime> usedAt(DateTime? value) =>
      _is.ColumnValue(
        table.usedAt,
        value,
      );
}

class MintChallengeTable extends _is.Table<_is.UuidValue?> {
  MintChallengeTable({super.tableRelation})
    : super(tableName: 'bitflip_mint_challenge') {
    updateTable = MintChallengeUpdateTable(this);
    walletAddress = _is.ColumnString(
      'walletAddress',
      this,
    );
    gameIndex = _is.ColumnInt(
      'gameIndex',
      this,
    );
    sectionIndex = _is.ColumnInt(
      'sectionIndex',
      this,
    );
    nonce = _is.ColumnString(
      'nonce',
      this,
    );
    message = _is.ColumnString(
      'message',
      this,
    );
    createdAt = _is.ColumnDateTime(
      'createdAt',
      this,
    );
    expiresAt = _is.ColumnDateTime(
      'expiresAt',
      this,
    );
    usedAt = _is.ColumnDateTime(
      'usedAt',
      this,
    );
  }

  late final MintChallengeUpdateTable updateTable;

  late final _is.ColumnString walletAddress;

  late final _is.ColumnInt gameIndex;

  late final _is.ColumnInt sectionIndex;

  late final _is.ColumnString nonce;

  late final _is.ColumnString message;

  late final _is.ColumnDateTime createdAt;

  late final _is.ColumnDateTime expiresAt;

  late final _is.ColumnDateTime usedAt;

  @override
  List<_is.Column> get columns => [
    id,
    walletAddress,
    gameIndex,
    sectionIndex,
    nonce,
    message,
    createdAt,
    expiresAt,
    usedAt,
  ];
}

class MintChallengeInclude extends _is.IncludeObject {
  MintChallengeInclude._();

  @override
  Map<String, _is.Include?> get includes => {};

  @override
  _is.Table<_is.UuidValue?> get table => MintChallenge.t;
}

class MintChallengeIncludeList extends _is.IncludeList {
  MintChallengeIncludeList._({
    _is.WhereExpressionBuilder<MintChallengeTable>? where,
    super.limit,
    super.offset,
    super.orderBy,
    super.orderByList,
    super.include,
  }) {
    super.where = where?.call(MintChallenge.t);
  }

  @override
  Map<String, _is.Include?> get includes => include?.includes ?? {};

  @override
  _is.Table<_is.UuidValue?> get table => MintChallenge.t;
}

class MintChallengeRepository {
  const MintChallengeRepository._();

  /// Returns a list of [MintChallenge]s matching the given query parameters.
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
  Future<List<MintChallenge>> find(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<MintChallengeTable>? where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<MintChallengeTable>? orderBy,
    _is.OrderByListBuilder<MintChallengeTable>? orderByList,
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.find<MintChallenge>(
      where: where?.call(MintChallenge.t),
      orderBy: orderBy?.call(MintChallenge.t),
      orderByList: orderByList?.call(MintChallenge.t),
      limit: limit,
      offset: offset,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Returns the first matching [MintChallenge] matching the given query parameters.
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
  Future<MintChallenge?> findFirstRow(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<MintChallengeTable>? where,
    int? offset,
    _is.OrderByBuilder<MintChallengeTable>? orderBy,
    _is.OrderByListBuilder<MintChallengeTable>? orderByList,
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.findFirstRow<MintChallenge>(
      where: where?.call(MintChallenge.t),
      orderBy: orderBy?.call(MintChallenge.t),
      orderByList: orderByList?.call(MintChallenge.t),
      offset: offset,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Finds a single [MintChallenge] by its [id] or null if no such row exists.
  Future<MintChallenge?> findById(
    _is.DatabaseSession session,
    _is.UuidValue id, {
    _is.Transaction? transaction,
    _is.LockMode? lockMode,
    _is.LockBehavior? lockBehavior,
  }) async {
    return session.db.findById<MintChallenge>(
      id,
      transaction: transaction,
      lockMode: lockMode,
      lockBehavior: lockBehavior,
    );
  }

  /// Inserts all [MintChallenge]s in the list and returns the inserted rows.
  ///
  /// The returned [MintChallenge]s will have their `id` fields set.
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
  Future<List<MintChallenge>> insert(
    _is.DatabaseSession session,
    List<MintChallenge> rows, {
    _is.Transaction? transaction,
    bool ignoreConflicts = false,
    bool noReturn = false,
  }) async {
    return session.db.insert<MintChallenge>(
      rows,
      transaction: transaction,
      ignoreConflicts: ignoreConflicts,
      noReturn: noReturn,
    );
  }

  /// Inserts a single [MintChallenge] and returns the inserted row.
  ///
  /// The returned [MintChallenge] will have its `id` field set.
  Future<MintChallenge> insertRow(
    _is.DatabaseSession session,
    MintChallenge row, {
    _is.Transaction? transaction,
  }) async {
    return session.db.insertRow<MintChallenge>(
      row,
      transaction: transaction,
    );
  }

  /// Upserts all [MintChallenge]s in the list and returns the resulting rows.
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
  /// The returned [MintChallenge]s will have their `id` fields set.
  ///
  /// This is an atomic operation, meaning that if one of the rows fails,
  /// none of the rows will be affected.
  ///
  /// If [noReturn] is set to `true`, the resulting rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<MintChallenge>> upsert(
    _is.DatabaseSession session,
    List<MintChallenge> rows, {
    required _is.ColumnSelections<MintChallengeTable> conflictColumns,
    _is.ColumnSelections<MintChallengeTable>? updateColumns,
    _is.WhereExpressionBuilder<MintChallengeTable>? updateWhere,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.upsert<MintChallenge>(
      rows,
      conflictColumns: conflictColumns(MintChallenge.t),
      updateColumns: updateColumns?.call(MintChallenge.t),
      updateWhere: updateWhere?.call(MintChallenge.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Upserts a single [MintChallenge] and returns the resulting row.
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
  /// The returned [MintChallenge] will have its `id` field set.
  Future<MintChallenge?> upsertRow(
    _is.DatabaseSession session,
    MintChallenge row, {
    required _is.ColumnSelections<MintChallengeTable> conflictColumns,
    _is.ColumnSelections<MintChallengeTable>? updateColumns,
    _is.WhereExpressionBuilder<MintChallengeTable>? updateWhere,
    _is.Transaction? transaction,
  }) async {
    return session.db.upsertRow<MintChallenge>(
      row,
      conflictColumns: conflictColumns(MintChallenge.t),
      updateColumns: updateColumns?.call(MintChallenge.t),
      updateWhere: updateWhere?.call(MintChallenge.t),
      transaction: transaction,
    );
  }

  /// Updates all [MintChallenge]s in the list and returns the updated rows. If
  /// [columns] is provided, only those columns will be updated. Defaults to
  /// all columns.
  /// This is an atomic operation, meaning that if one of the rows fails to
  /// update, none of the rows will be updated.
  ///
  /// If [noReturn] is set to `true`, the updated rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<MintChallenge>> update(
    _is.DatabaseSession session,
    List<MintChallenge> rows, {
    _is.ColumnSelections<MintChallengeTable>? columns,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.update<MintChallenge>(
      rows,
      columns: columns?.call(MintChallenge.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Updates a single [MintChallenge]. The row needs to have its id set.
  /// Optionally, a list of [columns] can be provided to only update those
  /// columns. Defaults to all columns.
  Future<MintChallenge> updateRow(
    _is.DatabaseSession session,
    MintChallenge row, {
    _is.ColumnSelections<MintChallengeTable>? columns,
    _is.Transaction? transaction,
  }) async {
    return session.db.updateRow<MintChallenge>(
      row,
      columns: columns?.call(MintChallenge.t),
      transaction: transaction,
    );
  }

  /// Updates a single [MintChallenge] by its [id] with the specified [columnValues].
  /// Returns the updated row or null if no row with the given id exists.
  Future<MintChallenge?> updateById(
    _is.DatabaseSession session,
    _is.UuidValue id, {
    required _is.ColumnValueListBuilder<MintChallengeUpdateTable> columnValues,
    _is.Transaction? transaction,
  }) async {
    return session.db.updateById<MintChallenge>(
      id,
      columnValues: columnValues(MintChallenge.t.updateTable),
      transaction: transaction,
    );
  }

  /// Updates all [MintChallenge]s matching the [where] expression with the specified [columnValues].
  /// Returns the list of updated rows.
  ///
  /// If [noReturn] is set to `true`, the updated rows are not read back from
  /// the database and an empty list is returned. This avoids the overhead of
  /// transferring and deserializing the rows when the result is not needed.
  Future<List<MintChallenge>> updateWhere(
    _is.DatabaseSession session, {
    required _is.ColumnValueListBuilder<MintChallengeUpdateTable> columnValues,
    required _is.WhereExpressionBuilder<MintChallengeTable> where,
    int? limit,
    int? offset,
    _is.OrderByBuilder<MintChallengeTable>? orderBy,
    _is.OrderByListBuilder<MintChallengeTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.updateWhere<MintChallenge>(
      columnValues: columnValues(MintChallenge.t.updateTable),
      where: where(MintChallenge.t),
      limit: limit,
      offset: offset,
      orderBy: orderBy?.call(MintChallenge.t),
      orderByList: orderByList?.call(MintChallenge.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Deletes all [MintChallenge]s in the list and returns the deleted rows.
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
  Future<List<MintChallenge>> delete(
    _is.DatabaseSession session,
    List<MintChallenge> rows, {
    _is.OrderByBuilder<MintChallengeTable>? orderBy,
    _is.OrderByListBuilder<MintChallengeTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.delete<MintChallenge>(
      rows,
      orderBy: orderBy?.call(MintChallenge.t),
      orderByList: orderByList?.call(MintChallenge.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Deletes a single [MintChallenge].
  Future<MintChallenge> deleteRow(
    _is.DatabaseSession session,
    MintChallenge row, {
    _is.Transaction? transaction,
  }) async {
    return session.db.deleteRow<MintChallenge>(
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
  Future<List<MintChallenge>> deleteWhere(
    _is.DatabaseSession session, {
    required _is.WhereExpressionBuilder<MintChallengeTable> where,
    _is.OrderByBuilder<MintChallengeTable>? orderBy,
    _is.OrderByListBuilder<MintChallengeTable>? orderByList,
    _is.Transaction? transaction,
    bool noReturn = false,
  }) async {
    return session.db.deleteWhere<MintChallenge>(
      where: where(MintChallenge.t),
      orderBy: orderBy?.call(MintChallenge.t),
      orderByList: orderByList?.call(MintChallenge.t),
      transaction: transaction,
      noReturn: noReturn,
    );
  }

  /// Counts the number of rows matching the [where] expression. If omitted,
  /// will return the count of all rows in the table.
  Future<int> count(
    _is.DatabaseSession session, {
    _is.WhereExpressionBuilder<MintChallengeTable>? where,
    int? limit,
    _is.Transaction? transaction,
  }) async {
    return session.db.count<MintChallenge>(
      where: where?.call(MintChallenge.t),
      limit: limit,
      transaction: transaction,
    );
  }

  /// Acquires row-level locks on [MintChallenge] rows matching the [where] expression.
  Future<void> lockRows(
    _is.DatabaseSession session, {
    required _is.WhereExpressionBuilder<MintChallengeTable> where,
    required _is.LockMode lockMode,
    required _is.Transaction transaction,
    _is.LockBehavior lockBehavior = _is.LockBehavior.wait,
  }) async {
    return session.db.lockRows<MintChallenge>(
      where: where(MintChallenge.t),
      lockMode: lockMode,
      lockBehavior: lockBehavior,
      transaction: transaction,
    );
  }
}
