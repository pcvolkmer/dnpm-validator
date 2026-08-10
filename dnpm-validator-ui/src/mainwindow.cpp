#include "mainwindow.h"
#include "ui_mainwindow.h"

MainWindow::MainWindow(QWidget* parent) :
    QMainWindow(parent), ui(new Ui::MainWindow)
{
    ui->setupUi(this);

    QFont monospaceFont("monospace");
    monospaceFont.setStyleHint(QFont::TypeWriter);

    this->ui->plainTextEdit->setFont(monospaceFont);
    this->ui->errorListWidget->setFont(monospaceFont);

    this->positionLabel = new QLabel(this);
    this->formatSelection = new QComboBox(this);
    this->formatSelection->addItem("DNPM Datenmodell 2.1");
    this->formatSelection->addItem("SE:dip Datenmodell");
    this->formatSelection->addItem("GRZ Metadata 1.3.1");
    this->ui->toolBar->addWidget(this->formatSelection);
    this->ui->statusbar->addPermanentWidget(this->positionLabel);

    connect(ui->actionOpen, &QAction::triggered, this, &MainWindow::onOpenAction);
    connect(ui->actionSave, &QAction::triggered, this, &MainWindow::onSaveAction);
    connect(ui->actionSaveAs, &QAction::triggered, this, &MainWindow::onSaveAsAction);
    connect(ui->actionValidate, &QAction::triggered, this, &MainWindow::onValidateAction);
    connect(this->formatSelection, &QComboBox::currentTextChanged, [this](const QString&)
    {
        this->onValidateAction();
    });
    connect(ui->errorListWidget, &QListWidget::itemClicked,
                  [this](const QListWidgetItem* item)
                  {
                      const auto row = item->listWidget()->currentIndex().row();
                      this->onErrorSelected(row);
                  });
    connect(ui->plainTextEdit, &QPlainTextEdit::cursorPositionChanged, [this]
    {
        this->positionLabel->setText(
            QString("[%1:%2]")
            .arg(ui->plainTextEdit->textCursor().blockNumber() + 1)
            .arg(ui->plainTextEdit->textCursor().columnNumber() + 1)
        );
    });
    connect(ui->actionAbout, &QAction::triggered, [this]
    {
        QMessageBox::about(this,
                           "About DNPM-Validator",
                           R"(
<html><body>
<p style="font-size: large; font-weight: bold;">DNPM-Validator</p>
<p>Application to validate and edit a data set in DNPM Datenmodell 2.1 and SE:dip data model format</p>
<p><a href="https://github.com/pcvolkmer/dnpm-validator">https://github.com/pcvolkmer/dnpm-validator</a></p>
</body></html>)"
        );
    });
}

MainWindow::~MainWindow()
{
    delete this->formatSelection;
    delete this->positionLabel;
    delete ui;
}

void MainWindow::onOpenAction()
{
    this->filename = QFileDialog::getOpenFileName(
        this,
        "Open file",
        QDir::homePath(),
        "JSON files (*.json);;All files (*.*)"
    );

    if (!this->filename.isEmpty())
    {
        QFile file(this->filename);
        if (file.open(QIODevice::ReadOnly | QIODevice::Text))
        {
            QByteArray content = file.readAll();

            ui->plainTextEdit->setPlainText(
                QString::fromUtf8(content)
            );

            this->onValidateAction();
        }
        this->setWindowTitle(QString("DNPM-Validator :: %1").arg(QFileInfo(this->filename).fileName()));
        return;
    }
    this->setWindowTitle("DNPM-Validator");
}

void MainWindow::onSaveAction()
{
    if (!this->filename.isEmpty())
    {
        QFile file(this->filename);
        if (file.open(QIODevice::WriteOnly | QIODevice::Text))
        {
            this->onValidateAction();
            file.write(this->ui->plainTextEdit->toPlainText().toUtf8());
            file.close();
        }
        return;
    }
    this->onSaveAsAction();
}

void MainWindow::onSaveAsAction()
{
    auto selectedFilename = QFileDialog::getSaveFileName(
        this,
        "Save file",
        QDir::homePath(),
        "JSON files (*.json);;All files (*.*)"
    );
    if (!selectedFilename.isEmpty())
    {
        this->filename = selectedFilename;
        this->onSaveAction();
        this->setWindowTitle(QString("DNPM-Validator :: %1").arg(QFileInfo(this->filename).fileName()));
        return;
    }
    this->setWindowTitle("DNPM-Validator");
}

void MainWindow::onValidateAction()
{
    auto json = ui->plainTextEdit->toPlainText();
    auto validationType = dnpmvalidation::ValidationType::Mtb;
    if (this->formatSelection->currentIndex() == 1)
    {
        validationType = dnpmvalidation::ValidationType::Rd;
    } else if (this->formatSelection->currentIndex() == 2) {
        validationType = dnpmvalidation::ValidationType::Grz;
    }
    auto errors = dnpmvalidation::validate(rust::String(json.toStdString()), validationType);

    this->errorList.clear();
    this->ui->errorListWidget->clear();

    if (errors.empty())
    {
        this->formatSelection->setStyleSheet("");
        this->markErrors();
        return;
    }

    for (auto error : errors)
    {
        this->errorList.push_back(error);
        ui->errorListWidget->addItem(
            QString("[%1:%2]   %3").arg(error.startLine, 4).arg(error.startColumn, 4).arg(error.message.c_str()));
    }

    this->formatSelection->setStyleSheet("background-color: rgba(200,0,0,80)");
    this->markErrors();
}

void MainWindow::onErrorSelected(int index)
{
    auto error = this->errorList.at(index);
    auto cursor = ui->plainTextEdit->textCursor();

    cursor.clearSelection();
    cursor.setPosition(
        QTextCursor::Start
    );

    auto startTextBlock = ui->plainTextEdit->document()->findBlockByLineNumber(error.startLine - 1);
    cursor.setPosition(startTextBlock.position() + error.startColumn - 1, QTextCursor::MoveAnchor);
    ui->plainTextEdit->setTextCursor(cursor);
    ui->plainTextEdit->ensureCursorVisible();
}

void MainWindow::markErrors()
{
    auto cursor = ui->plainTextEdit->textCursor();

    QList<QTextEdit::ExtraSelection> extraSelections;

    for (const auto& error : this->errorList)
    {
        auto startTextBlock = ui->plainTextEdit->document()->findBlockByLineNumber(error.startLine - 1);
        cursor.setPosition(startTextBlock.position() + error.startColumn - 1, QTextCursor::MoveAnchor);

        auto endTextBlock = ui->plainTextEdit->document()->findBlockByLineNumber(error.endLine - 1);
        cursor.setPosition(endTextBlock.position() + error.endColumn - 1, QTextCursor::KeepAnchor);

        QTextEdit::ExtraSelection selection;
        selection.cursor = cursor;
        QTextCharFormat format;
        format.setUnderlineStyle(QTextCharFormat::WaveUnderline);
        format.setUnderlineColor(Qt::red);
        selection.format = format;
        extraSelections.append(selection);
    }

    ui->plainTextEdit->setExtraSelections(extraSelections);
}
